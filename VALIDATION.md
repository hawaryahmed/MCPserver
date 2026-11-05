# v0.5.1 — quiet assessment workflow

- Removed all automatic updateModelContext calls from create, save, completion, resume, restore and conflict recovery.
- The explicit Summarize in chat button sends one short request containing the assessment ID and organization; no raw report JSON. It is disabled after a successful request for the same report revision.
- Browser regression test asserts zero ui/update-model-context events through the entire workflow and exactly one ui/message, triggered by the summary button. It also verifies the message contains no report JSON.
- Existing assessment, storage, scoring, versioning and Free/native Rust configuration are unchanged. UI resource URI changed to prevent reusing the old cached template.
- Real ChatGPT display after deployment still requires merge/deploy and app refresh.

Local verification: locked Rust debug build and the complete headless browser regression passed. GitHub Actions runs the full Rust/HTTP/release checks as well.
