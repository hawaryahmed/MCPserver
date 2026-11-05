# v0.3.1 verification — 2026-11-05

- Five Rust unit tests passed for the anonymous demo changes.
- Locked debug build passed.
- Live local HTTP smoke test passed without an Authorization header or MCP_API_KEY: initialization, all six noauth tool declarations, concurrent answer saves, scoring, listing and deletion, notification 202, protocol/Origin checks, and process restart file handling.
- Render YAML parses, selects plan free, and declares no disk, database or API key.
- Docker build and deployment checks run in GitHub Actions. Live Render configuration and billing require checking the service dashboard; changing YAML alone does not confirm the existing service plan changed.
- No claim that the deployed service has updated until its health response reports 0.3.1.
