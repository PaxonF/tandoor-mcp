# Tandoor MCP Server

Exposes your Tandoor Recipes instance to MCP clients such as Claude over the
Streamable HTTP transport. The MCP endpoint is `http://<home-assistant-host>:3001/mcp`.

## Configuration

| Option               | Description                                                              |
| -------------------- | ------------------------------------------------------------------------ |
| `tandoor_url`        | Full URL of your Tandoor instance, e.g. `http://192.168.1.10:8080`       |
| `tandoor_auth_token` | Tandoor API token (**preferred**)                                        |
| `tandoor_username`   | Username, only used when no token is set                                 |
| `tandoor_password`   | Password, only used when no token is set                                 |
| `log_level`          | `trace`, `debug`, `info`, `warn` or `error`                              |

Prefer a token: Tandoor limits its login endpoint to 10 requests per day per IP.
Create one in Tandoor under **Settings → API**, or:

```bash
curl -X POST http://your-tandoor/api-token-auth/ \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
```

If Tandoor runs as another Home Assistant app, use its hostname or your Home
Assistant host's IP rather than `localhost`, which points at this app's container.

To change the host port, edit the **Network** section of this app.

## Exposing to Claude

Claude custom connectors need a public HTTPS URL. Put the endpoint behind a
reverse proxy (or a tunnel) and add a query-key gate; see the
[project README](https://github.com/PaxonF/tandoor-mcp#securing-the-connector-with-a-query-key-gate-recommended).
