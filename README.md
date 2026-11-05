# SME Cyber Maturity MCP — public demo v0.3.1

Independent ENISA-informed assessment engine. Not an official ENISA product or grading system.

## Render Free configuration

`render.yaml` provisions one Docker web service with `plan: free`: **$0/month base compute, 0.1 CPU, 512 MB RAM**. CPU and RAM come from the plan; do not add unsupported CPU/RAM YAML fields. No persistent disk, database, or other paid resource is declared. Render bandwidth/build usage limits still apply.

Assessment files use the ephemeral `/var/data` directory. Records are lost on restart, redeploy, or idle spin-down. Free services spin down after 15 minutes of inactivity and may take about a minute to wake up. Open `/healthz` and wait before connecting ChatGPT after an idle period.

## Update your existing deployment

1. Merge this change and deploy the latest `main` commit in Render.
2. If managed by a Blueprint, sync the Blueprint. If created manually, set the service's Instance Type to **Free** in Render; a YAML file alone does not change a manually configured service.
3. If the old paid deployment has a disk, export any records you want to keep before detaching it. Free services cannot have persistent disks. Removing the disk block from YAML does not guarantee Render deletes an existing disk or stops its charges. Check the Render dashboard and remove the old disk yourself after backup.
4. Remove the old `MCP_API_KEY` environment variable if desired. The demo ignores it and never requires a key.
5. Set `MCP_ALLOWED_ORIGINS=https://chatgpt.com` (included in the Blueprint). Requests without an Origin header are accepted; unexpected Origin headers remain blocked.
6. Verify `/healthz` returns version `0.3.1`. Until then, the older authenticated version may still be running.

## Connect through ChatGPT

Server URL: `https://sme-cyber-maturity-mcp.onrender.com/mcp`

Choose **No Authentication**, not OAuth. Recreate or update the connection if ChatGPT cached the old OAuth setting. No client ID, client secret, API key, or OAuth discovery endpoint is needed. All six tools declare `securitySchemes: [{"type":"noauth"}]`.

This is a public, shared demo. Anyone reaching it can list, read, change and delete assessments. Use fictitious data only. There is no user/tenant isolation, OAuth, application audit trail or rate limit. All six tools remain available for demonstration.

## Endpoints

- GET `/healthz`: public status and version.
- POST `/mcp`: anonymous JSON-RPC; notifications return empty HTTP 202.
- GET/DELETE `/mcp`: HTTP 405, because this stateless server does not provide an SSE stream or sessions. Opening `/mcp` in a browser is not a functionality test.
- Supported MCP versions: `2025-03-26`, `2025-06-18`, `2025-11-25`.
- Requests require Content-Type `application/json` and Accept `application/json, text/event-stream`.
- Send `MCP-Protocol-Version` after initialization. The request limit is 256 KiB.

```sh
curl https://sme-cyber-maturity-mcp.onrender.com/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

## Development and checks

```sh
cargo test --locked --manifest-path server/Cargo.toml
cargo build --locked --manifest-path server/Cargo.toml
ENISA_MCP_DATA_DIR=./data server/target/debug/enisa-sme-mcp
```

Run `python3 tests/smoke.py` to launch a temporary local server and test the anonymous assessment workflow. For remote verification: `python3 tests/smoke.py https://sme-cyber-maturity-mcp.onrender.com`. The test creates and deletes its own assessment. Restart persistence in the local test checks file handling only; it does not imply durability on Render Free.

For stdio clients use `server/target/debug/enisa-sme-mcp --stdio`. For Docker run `docker build -t sme-mcp .` then `docker run --rm -p 127.0.0.1:10000:10000 sme-mcp`. No secret is needed.

Tools: `get_question_bank`, `create_assessment`, `record_answer`, `calculate_maturity`, `list_assessments`, `delete_assessment`. Scoring is independent and unchanged. Evidence references are self-reported and are not proof of verification.

## References

- https://render.com/docs/compute-plans
- https://render.com/docs/free
- https://render.com/docs/blueprint-spec
- https://developers.openai.com/plugins/build/auth
- https://modelcontextprotocol.io/specification/2025-11-25/basic/transports

See `VALIDATION.md` for actual verification results.
