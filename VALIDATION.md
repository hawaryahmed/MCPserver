# v0.5.0 verification — 2026-11-05

- 13 Rust tests passed, including 22 score/coverage/maturity/priority fixtures captured from the previous v0.4.0 executable, frozen question wording/guidance, legacy-record migration, single-call saves, server validation, completion/handoff, evidence-confidence separation and backup/restore.
- A real HTTP concurrency test sends two writes with the same revision: exactly one is applied, the other returns revision_conflict, and storage contains one update.
- Locked debug and optimized release builds passed.
- Existing live HTTP smoke test passed with the new tool count and backward-compatible revision omission, including concurrent saves and process restart file handling.
- Headless Chromium passed through a sandboxed cross-origin MCP Apps bridge using the actual Rust server and bundled widget. Coverage includes overview/create, one-question flow, conditional follow-ups, NA validation, save/back/edit, completion score, completed-resume behavior, one-call saves (no full get_assessment after each save), completion context and chat-message handoff, export/download/restore, stale-edit rejection/reload/draft recovery, authored remediation display and mobile overflow.
- Skill references are generated from the versioned JSON bank. CI checks generated references and the bundled HTML for drift.
- Native Rust / Render Free configuration is retained; no new paid resource, authentication or durable storage was added.
- Real ChatGPT client download/message support and live Render deployment remain to be verified after merging/deploying and refreshing the app connection. The host-bridge test does not certify every ChatGPT client.
- Old unversioned records default to bank 1.0.0 and revision 0. Text is retained on read. New writes/imports enforce documented limits; unusually large legacy records need shortened content to restore under those limits.
- Revision is optional for legacy clients; stale-write protection applies when clients supply expected_revision. Storage remains single-process.
