# SME Cyber Maturity MCP — Render edition v0.3.0

Independent ENISA-informed assessment engine. Not an official ENISA product or grading system.
Rust + Axum, stateless Streamable HTTP with JSON responses, plus optional stdio.

## Deploy to Render

1. Extract this directory. Upload **its contents** to the root of a new private GitHub repository. Include `.github`, `Dockerfile`, `render.yaml`, `server/Cargo.lock`, and all source files. Do not upload the ZIP itself.
2. In Render, choose **New → Blueprint**, connect that repository and select `render.yaml`.
3. Review the paid Starter web service and 1 GB persistent disk, then deploy. This package has not authorized or created a Render purchase for you.
4. After deployment, open the service's Environment settings. Copy the generated `MCP_API_KEY` privately into your MCP client's secret configuration.
5. Check `https://YOUR-SERVICE.onrender.com/healthz`; it should return status `ok`.
6. Your MCP URL is `https://YOUR-SERVICE.onrender.com/mcp`. Send `Authorization: Bearer <MCP_API_KEY>` on every MCP request. Never place the key in the URL.
7. Run `python3 tests/smoke.py https://YOUR-SERVICE.onrender.com` with `MCP_API_KEY` set locally. It creates and deletes its own test assessment.

Render sets up HTTPS. `/healthz` is public and exposes no assessments. `/mcp` requires the token, including GET and DELETE. The disk is required: filesystem changes outside it are ephemeral. Render disks require paid services and constrain deployment to one instance. Review current pricing in Render before deploying.

If startup says storage is not writable, check ownership of `/var/data` for UID/GID 10001 in the Render shell. Do not remove the persistent disk to fix this: that would remove the durability guarantee.

## Client connection

Use a client that supports Streamable HTTP plus a custom bearer Authorization header. Configure the URL and secret through that client's documented settings; JSON config formats differ among clients.

**This is private shared-key authentication, not an OAuth implementation.** It does not supply OAuth discovery, registration, authorization or token endpoints. It is not a promise of automatic compatibility with ChatGPT's connector authentication flow. The existing ChatGPT plugin has not been modified or connected. For a host requiring OAuth, place this behind a compatible OAuth resource gateway and validate resource-bound tokens there, or implement OAuth before connecting. Do not turn off authentication to work around this.

All token holders share the same assessments and can list, modify and delete them. Deploy one instance per trusted workspace. This is not a multi-tenant SaaS backend. No frontend or evidence upload service is included; evidence fields store references only.

## Transport and tools

- POST `/mcp`: JSON-RPC requests; notifications receive empty HTTP 202.
- Supported MCP revisions: `2025-03-26`, `2025-06-18`, `2025-11-25`. Initialize negotiates a supported version.
- Requests require Content-Type `application/json` and Accept `application/json, text/event-stream`.
- Send `MCP-Protocol-Version` after initialization. Unsupported versions receive HTTP 400.
- No sessions, server-initiated messages or SSE stream. Authenticated GET/DELETE return 405.
- Origin headers are rejected unless explicitly allowlisted in comma-separated `MCP_ALLOWED_ORIGINS`. Headerless server clients are accepted with the token. No browser CORS integration is supplied.
- 256 KiB request limit; all storage operations serialized within one process; atomic file replacement.

Tools: `get_question_bank`, `create_assessment`, `record_answer`, `calculate_maturity`, `list_assessments`, `delete_assessment`.

Scores and priorities are unchanged from v0.2.0. Evidence references are self-reported, not proof of verification. A 30/60/90-day plan is a template requiring business-context review.

## Local development

```sh
cargo test --locked --manifest-path server/Cargo.toml
cargo build --locked --manifest-path server/Cargo.toml
# Set MCP_API_KEY in your shell; use a secret manager or a random 32+ char value.
ENISA_MCP_DATA_DIR=./data server/target/debug/enisa-sme-mcp
```

In another shell with the same key: `python3 tests/smoke.py`.
For legacy local MCP clients use `server/target/debug/enisa-sme-mcp --stdio` (no HTTP token required for stdio). `mcp.json` retains the local plugin configuration with this argument.

Docker: `docker build -t sme-mcp .`, then run with `-p 127.0.0.1:10000:10000 -e MCP_API_KEY -v sme-mcp-data:/var/data`. Only one process may use the data volume. Do not run HTTP and stdio concurrently against the same directory.

## Operations

Back up the mounted data directory, protect backup access, and test restoration. Rotate the token by replacing `MCP_API_KEY` in Render and updating authorized clients. Redeploys with disks may interrupt availability. Use a database and tenant authorization before horizontal scaling or customer self-service. No application audit trail, per-user authorization, quota or rate limiting is included; put limits at your gateway before broader exposure. Do not store secrets or raw sensitive evidence in answer fields.

## References

- Render Blueprint: https://render.com/docs/blueprint-spec
- Render Docker deployment: https://render.com/docs/docker
- Render persistent disks: https://render.com/docs/disks
- MCP transport: https://modelcontextprotocol.io/specification/2025-11-25/basic/transports
- ENISA SME guidance: https://www.enisa.europa.eu/publications/enisa-report-cybersecurity-for-smes

See `VALIDATION.md` for the actual checks performed on this package.
