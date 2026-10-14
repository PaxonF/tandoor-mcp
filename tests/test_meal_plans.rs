mod common;

use common::{DockerEnvironment, TestEnvironment};
use mcp_tandoor::client::types::CreateMealPlanRequest;
use serial_test::serial;

fn date(s: &str) -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

/// A fresh Tandoor has no meal types, and the client has no endpoint for creating
/// one, so create it directly. Tandoor get-or-creates meal types by name.
async fn ensure_meal_type(env: &TestEnvironment) -> i32 {
    let base_url =
        std::env::var("TANDOOR_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let token = env.client.get_token().expect("test client has a token");
    let meal_type: serde_json::Value = reqwest::Client::new()
        .post(format!("{base_url}/api/meal-type/"))
        .bearer_auth(token)
        .json(&serde_json::json!({"name": "MCP Test Dinner"}))
        .send()
        .await
        .expect("create meal type")
        .error_for_status()
        .expect("meal type accepted")
        .json()
        .await
        .expect("meal type json");
    meal_type["id"].as_i64().expect("meal type id") as i32
}

#[tokio::test]
#[serial]
async fn test_meal_plan_lifecycle() {
    common::init_test_logging();
    DockerEnvironment::ensure_running().expect("Docker environment not running");
    let env = TestEnvironment::new()
        .await
        .expect("Failed to create test environment");

    let meal_type = ensure_meal_type(&env).await;

    let meal_types = env.client.get_meal_types().await.expect("get meal types");
    assert!(meal_types.results.iter().any(|mt| mt.id == meal_type));

    // Title-only, multi-day plan with no note: optional fields must be omitted, not null
    let created = env
        .client
        .create_meal_plan(CreateMealPlanRequest {
            recipe: None,
            title: Some("MCP lifecycle test".to_string()),
            servings: 2.5,
            from_date: date("2031-01-15"),
            to_date: date("2031-01-16"),
            meal_type,
            note: None,
        })
        .await
        .expect("create meal plan");
    assert_eq!(created.date(), "2031-01-15");
    assert_eq!(created.end_date(), "2031-01-16");
    assert_eq!(created.servings, 2.5);

    let listed = env
        .client
        .get_meal_plans(Some("2031-01-01"), Some("2031-01-31"))
        .await
        .expect("get meal plans with results");
    let plan = listed
        .results
        .iter()
        .find(|p| p.id == created.id)
        .expect("created plan is listed");
    assert_eq!(plan.meal_type.id, meal_type);

    let updated = env
        .client
        .update_meal_plan(
            created.id,
            serde_json::json!({"from_date": "2031-01-20", "to_date": "2031-01-20"}),
        )
        .await
        .expect("update meal plan");
    assert_eq!(updated.date(), "2031-01-20");
    assert_eq!(updated.end_date(), "2031-01-20");

    env.client
        .delete_meal_plan(created.id)
        .await
        .expect("delete meal plan");
}

#[tokio::test]
#[serial]
async fn test_meal_plan_validation_error_includes_tandoor_details() {
    common::init_test_logging();
    DockerEnvironment::ensure_running().expect("Docker environment not running");
    let env = TestEnvironment::new()
        .await
        .expect("Failed to create test environment");

    let meal_type = ensure_meal_type(&env).await;
    let created = env
        .client
        .create_meal_plan(CreateMealPlanRequest {
            recipe: None,
            title: Some("MCP validation test".to_string()),
            servings: 1.0,
            from_date: date("2031-02-01"),
            to_date: date("2031-02-01"),
            meal_type,
            note: None,
        })
        .await
        .expect("create meal plan");

    let err = env
        .client
        .update_meal_plan(created.id, serde_json::json!({"servings": "lots"}))
        .await
        .expect_err("non-numeric servings is rejected");
    let message = err.to_string();
    assert!(message.contains("400"), "{message}");
    assert!(
        message.contains("servings"),
        "error should carry Tandoor's field details: {message}"
    );

    env.client
        .delete_meal_plan(created.id)
        .await
        .expect("delete meal plan");
}
