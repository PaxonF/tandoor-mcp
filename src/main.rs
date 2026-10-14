//! # Tandoor MCP Server
//!
//! A Model Context Protocol (MCP) server that provides tools for interacting with Tandoor,
//! a recipe management system. This server allows AI assistants to search recipes, create
//! new recipes, manage shopping lists, and more through a standardized protocol.
//!
//! ## Environment Variables
//!
//! - `TANDOOR_BASE_URL`: Tandoor server URL (default: http://localhost:8080)
//! - `TANDOOR_AUTH_TOKEN`: Pre-existing Tandoor API token (preferred; avoids rate-limited login)
//! - `TANDOOR_USERNAME`: Tandoor username (used only when TANDOOR_AUTH_TOKEN is not set)
//! - `TANDOOR_PASSWORD`: Tandoor password (used only when TANDOOR_AUTH_TOKEN is not set)
//! - `BIND_ADDR`: Address to bind the MCP server (default: 0.0.0.0:3001)
//! - `RUST_LOG`: Logging level (info, debug, trace, etc.)
//!
//! ## Home Assistant
//!
//! When run as a Home Assistant app, Supervisor writes the Configuration tab to
//! `/data/options.json`. Any non-empty option there takes precedence over the matching
//! environment variable (see [`HA_OPTION_ENV`]).

use mcp_tandoor::server::TandoorMcpServer;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use serde_json::{Map, Value};
use std::env;
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Where Home Assistant Supervisor writes an app's Configuration tab.
const HA_OPTIONS_PATH: &str = "/data/options.json";

/// Home Assistant option keys and the environment variables they override.
const HA_OPTION_ENV: &[(&str, &str)] = &[
    ("tandoor_url", "TANDOOR_BASE_URL"),
    ("tandoor_auth_token", "TANDOOR_AUTH_TOKEN"),
    ("tandoor_username", "TANDOOR_USERNAME"),
    ("tandoor_password", "TANDOOR_PASSWORD"),
    ("log_level", "RUST_LOG"),
];

/// Settings read from Home Assistant options first, then the environment.
struct Settings {
    ha_options: Map<String, Value>,
}

impl Settings {
    fn load() -> anyhow::Result<Self> {
        let path = Path::new(HA_OPTIONS_PATH);
        let ha_options = if path.exists() {
            let raw = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read {HA_OPTIONS_PATH}: {e}"))?;
            Self::parse_ha_options(&raw)?
        } else {
            Map::new()
        };
        Ok(Self { ha_options })
    }

    fn parse_ha_options(raw: &str) -> anyhow::Result<Map<String, Value>> {
        match serde_json::from_str(raw)? {
            Value::Object(map) => Ok(map),
            _ => anyhow::bail!("{HA_OPTIONS_PATH} is not a JSON object"),
        }
    }

    fn has_ha_options(&self) -> bool {
        !self.ha_options.is_empty()
    }

    /// Look up an environment variable, preferring its Home Assistant option when set.
    fn get(&self, env_key: &str) -> Option<String> {
        HA_OPTION_ENV
            .iter()
            .find(|(_, env)| *env == env_key)
            .and_then(|(opt, _)| self.ha_options.get(*opt))
            .and_then(|v| match v {
                Value::String(s) => Some(s.trim().to_string()),
                Value::Null => None,
                other => Some(other.to_string()),
            })
            .filter(|s| !s.is_empty())
            .or_else(|| env::var(env_key).ok().filter(|s| !s.is_empty()))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;

    tracing_subscriber::registry()
        .with(
            settings
                .get("RUST_LOG")
                .and_then(|filter| tracing_subscriber::EnvFilter::try_new(filter).ok())
                .unwrap_or_else(|| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if settings.has_ha_options() {
        tracing::info!("Loaded configuration from Home Assistant ({HA_OPTIONS_PATH})");
    }

    let base_url = settings
        .get("TANDOOR_BASE_URL")
        .unwrap_or_else(|| "http://localhost:8080".to_string());
    let bind_addr = settings
        .get("BIND_ADDR")
        .unwrap_or_else(|| "0.0.0.0:3001".to_string());

    // Authenticate: prefer TANDOOR_AUTH_TOKEN to avoid the 10-req/day rate limit on
    // the /api-token-auth/ login endpoint.
    tracing::info!("Validating Tandoor credentials...");
    let startup_server = if let Some(token) = settings.get("TANDOOR_AUTH_TOKEN") {
        tracing::info!("Using pre-existing auth token from TANDOOR_AUTH_TOKEN");
        let server = TandoorMcpServer::new(base_url.clone());
        server.set_global_auth_token(token).await?;
        server
    } else {
        let username = settings
            .get("TANDOOR_USERNAME")
            .unwrap_or_else(|| "admin".to_string());
        let password = settings
            .get("TANDOOR_PASSWORD")
            .unwrap_or_else(|| "admin".to_string());
        let server = TandoorMcpServer::new_with_credentials(
            base_url.clone(),
            username.clone(),
            password.clone(),
        );
        if let Err(e) = server
            .authenticate(username.clone(), password.clone())
            .await
        {
            tracing::error!("Authentication failed: {}", e);
            tracing::error!("  - TANDOOR_BASE_URL: {}", base_url);
            tracing::error!("  - TANDOOR_USERNAME: {}", username);
            tracing::error!("  - Set TANDOOR_AUTH_TOKEN to bypass login rate limiting");
            std::process::exit(1);
        }
        server
    };

    tracing::info!("Testing API access with token...");
    match startup_server.test_api_access().await {
        Ok(_) => tracing::info!("API access test passed"),
        Err(e) => {
            tracing::warn!("API access test failed: {e}");
            tracing::warn!("The server will continue, but some tools may not work.");
        }
    }

    tracing::info!("Successfully authenticated with Tandoor");

    let base_url_clone = base_url.clone();

    let service = StreamableHttpService::new(
        move || Ok(TandoorMcpServer::new(base_url_clone.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Tandoor MCP Server listening on {bind_addr} (Streamable HTTP at /mcp)");

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap();
            tracing::info!("Shutting down...");
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(raw: &str) -> Settings {
        Settings {
            ha_options: Settings::parse_ha_options(raw).unwrap(),
        }
    }

    #[test]
    fn ha_option_overrides_env() {
        let s = settings(r#"{"tandoor_url": "http://tandoor.local:8080"}"#);
        assert!(s.has_ha_options());
        assert_eq!(
            s.get("TANDOOR_BASE_URL").as_deref(),
            Some("http://tandoor.local:8080")
        );
    }

    #[test]
    fn empty_or_missing_ha_option_falls_back_to_env() {
        let s = settings(r#"{"tandoor_auth_token": "  ", "tandoor_password": null}"#);
        assert_eq!(
            s.get("TANDOOR_AUTH_TOKEN"),
            env::var("TANDOOR_AUTH_TOKEN").ok()
        );
        assert_eq!(s.get("TANDOOR_PASSWORD"), env::var("TANDOOR_PASSWORD").ok());
    }

    #[test]
    fn rejects_non_object_options() {
        assert!(Settings::parse_ha_options("[]").is_err());
    }
}
