# SME Cyber Maturity MCP — v0.5.1

Independent ENISA-informed self-assessment. Not an official ENISA product, score, audit or certification.

## Deployment

`render.yaml` remains a native Rust web service on **Free: $0/month base compute, 0.1 CPU, 512 MB RAM**. It provisions no paid disk or database. Render usage limits apply.

| Setting | Value |
| --- | --- |
| Runtime | `rust` |
| Plan | `free` |
| Build | `cargo build --release --locked --manifest-path server/Cargo.toml` |
| Start | `./server/target/release/enisa-sme-mcp` |
| Host / port | `HOST=0.0.0.0`, `PORT=10000` |
| Storage | `ENISA_MCP_DATA_DIR=./data` |
| Allowed browser origin | `MCP_ALLOWED_ORIGINS=https://chatgpt.com` |
| Health check | `/healthz` (version `0.5.1`) |

Connect ChatGPT to `https://sme-cyber-maturity-mcp.onrender.com/mcp` using **No Authentication**. Merge/deploy the update and refresh the app's tools before using new actions. A manually configured Render service must independently be set to Rust / Free; repository YAML does not prove its current runtime or billing plan.

This is an anonymous shared demo. Anyone reaching the service can access, modify or delete assessments. Use fictional data only. Storage resets on restarts, redeploys and idle spin-down. Free services sleep after inactivity and can take about a minute to wake up. Open `/healthz` before a demonstration after idle time. Back up any existing paid disk before removing it; YAML changes do not guarantee existing disk charges stop.

## Assessment workflow

Open **Assessment overview**, choose scope and start. Each question includes answer text, implementation level, explanation, example, evidence reference, owner and a conditional follow-up. The follow-up changes for partial/missing, consistent/tested, Unknown and NA answers. Back/edit and resume are supported.

Each assessment is pinned to a questionnaire version. Completion means every scoped question has an answer, including Unknown or justified NA; it does not mean every control is implemented or verified. Completed assessments reopen their report. Scores, coverage and evidence confidence are separate.

**Summarize in chat** sends one short request naming the selected assessment ID. ChatGPT retrieves the saved report through the assessment tools. The form never calls updateModelContext: saving, completing, resuming, restoring and conflict recovery do not generate context attachments. The button is disabled after a successful request for that report revision to prevent accidental duplicate messages. No raw report JSON is inserted into a visible chat message. If a host does not support chat messages, use the ID to ask for a summary manually.

Recommendations include a specific action, owner role, effort, dependencies, expected outcome and verification evidence. Priorities and target dates are unchanged from the prior method; review them against actual business risk.

## Single source of assessment content

`assessment/banks/1.0.0.json` owns questions, domain/mode membership, explanations, examples, conditional follow-ups, score labels, scoring policy and remediation metadata. Rust parses and validates it once at startup, with the file embedded into the executable. Content edits need a rebuild/redeploy but no change to scoring logic.

Do not modify a released bank in place. For a questionnaire or scoring change, add a new versioned bank and register it in `content.rs`, retaining existing versions. New assessments select the latest bank; existing assessments use their stored `bank_version`. Unknown versions are rejected rather than silently rescored. The typed score contract remains 0–4 / U / NA.

Unversioned v0.4.0 records are read as bank `1.0.0`, revision `0`. Their answers remain unchanged. Version/revision fields are written on the next successful save. Existing records remain readable with their original text; new writes/imports enforce field and size limits.

The human-readable skill references are **generated** from the bank:

```sh
node assessment/tools/generate-references.mjs
```

CI checks that references and the bundled widget match their sources. Skills describe the workflow and obtain questionnaire content through MCP. `chatgpt-plugin/skills/` holds reusable instructions; preserve the target installed plugin's own generated manifest and App binding when syncing those skills. Do not replace another connection's identity with a checked-in App ID.

## Rust modules

| Module | Responsibility |
| --- | --- |
| `main.rs` | Startup and stdio entry point |
| `model.rs` | Typed requests, records, score/mode enums and validation |
| `content.rs` | Versioned bank registry and startup validation |
| `assessment.rs` | Assessment lifecycle, saves, progress and backup/restore |
| `scoring.rs` | Deterministic scores, gaps and completion handoff |
| `storage.rs` | Atomic JSON file persistence |
| `mcp.rs` | Tool schemas/metadata and JSON-RPC routing |
| `http.rs` | HTTP transport, Origin checks, limits and serialized access |
| `ui.rs` | Embedded widget resource and overview presentation |

## Tools and compatibility

Read actions: `assessment_overview`, `get_assessment`, `get_question_bank`, `calculate_maturity`, `list_assessments`, `export_assessment`.

Write actions: `create_assessment`, `record_answer`, `delete_assessment`, `import_assessment`.

`create_assessment` returns initial form data, including `assessment_id`, `bank_version` and `revision`. `record_answer` returns the saved answer, new revision, progress and status in one call; the final answer also returns `report` and `handoff`. The widget does not re-fetch the whole questionnaire after every save.

Send `expected_revision` when saving. If it is stale, the server returns an MCP tool error with `structuredContent.error.code = "revision_conflict"` and does not modify the record. The form keeps the local draft and offers **Reload saved version (keep draft)** for review and retry. Omitting the revision remains accepted for legacy clients, so those clients do not receive stale-write protection. All storage operations remain serialized within one process; do not run multiple processes against the same directory.

Typed Rust inputs generate tool JSON Schemas. Invalid scores/types, unexpected fields, unjustified NA and oversized text are rejected on the server. New answer limits: answer text 12,000 characters, follow-up 6,000, evidence/NA justification 4,000 each, owner 160, legacy date 80. The total saved record limit is 128 KiB and the HTTP request limit is 256 KiB. Old records above these limits remain readable/exportable but require shorter content before saving/restoring under the demo limits.

An evidence reference or caller-supplied `verified_at` date never grants a verified status. This demo reports `evidence_confidence: self_reported` and `independently_verified: 0`.

## Backup and restore

Use **Export backup**, then **Download JSON**. Downloads use the MCP Apps host bridge; if the host lacks support or the user cancels, the backup JSON remains visible for copying. Keep it outside the ephemeral service.

Use **Restore a backup** on the overview to choose an exported JSON file or paste its contents. `import_assessment` validates the format, bank version, question IDs, answers and limits, then assigns a **new ID** and revision 0. It never overwrites the source record. Backups contain the answers and question-bank version; they are not proof of independent verification. There is no encrypted backup or authentication in this demo.

## Development and tests

```sh
cargo test --locked --manifest-path server/Cargo.toml
cargo build --locked --manifest-path server/Cargo.toml
ENISA_MCP_DATA_DIR=./data server/target/debug/enisa-sme-mcp
```

For stdio: `server/target/debug/enisa-sme-mcp --stdio`.

The widget is prebundled HTML/CSS/JavaScript in `web/dist/assessment.html`, embedded in Rust. Node and browser tooling are development/CI-only; Render needs no Node or Python runtime.

```sh
cd web
npm ci
npm run build
npx playwright install chromium --only-shell
npm test
```

The browser test uses a sandboxed cross-origin MCP Apps bridge and the real Rust backend. It checks the workflow, one-call saves, quiet saves and explicit summary handoff, backup download/restore and stale-edit recovery. This does not certify real ChatGPT behavior on every client.

`server/tests/fixtures/legacy-scoring-v0.4.0.json` contains 22 results captured from the previous executable. Tests preserve scores, coverage, maturity bands, unknown/NA handling, priorities and target dates. Existing HTTP smoke tests also remain available with `python3 tests/smoke.py`; Python is test-only.

## References

- https://render.com/docs/compute-plans
- https://render.com/docs/free
- https://developers.openai.com/plugins/build/chatgpt-ui
- https://www.enisa.europa.eu/publications/enisa-report-cybersecurity-for-smes
