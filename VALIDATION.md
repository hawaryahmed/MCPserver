# v0.4.0 verification — 2026-11-05

- Eleven Rust unit tests passed, including 30 question-specific guides, overview-before-creation, UI resource registration, old-answer compatibility, new answer fields, and read/write action metadata.
- Locked debug build and live anonymous HTTP smoke test passed, including concurrency and restart file handling.
- Headless Chromium test passed through a sandboxed cross-origin MCP Apps host bridge with the actual bundled widget and Rust backend: overview, creation, single-question form, explanations/examples, NA validation, follow-up save, Back/edit, completion score, resume, mobile layout and no JavaScript errors.
- Mobile question layout visually inspected. Browser testing caught and fixed native form submission restrictions in sandboxed app frames; Save & Next now uses the tool bridge without an HTML form submission.
- Production remains native Rust on Render Free; the UI is prebundled HTML/CSS/JavaScript embedded into the executable. Node tooling is development/CI-only.
- Connected plugin updated separately with three skills; existing App binding and presentation preserved. Repository source includes the same package for review.
- Real ChatGPT iframe behavior after deployment, mobile-specific host behavior, and Render's live deployment remain unverified until the PR is merged/deployed and the connection is refreshed.
- Render Free storage remains ephemeral and public. No OAuth, tenant isolation or durable storage was added.
