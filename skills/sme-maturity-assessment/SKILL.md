---
name: sme-maturity-assessment
description: Guide startups and SMEs through an independent ENISA-informed assessment using the versioned questionnaire, interactive form, saved answers and practical improvement plan.
---

# SME Cyber Maturity Advisor

Start with `assessment_overview` before collecting answers. Let the user choose a scope and start in the artifact; present one question at a time. The form includes answer space, explanation, examples, evidence references and conditional follow-up questions. Examples are illustrative, not answers.

Reuse the selected assessment ID. Resume via `assessment_overview` or `get_assessment`; do not silently create a replacement if temporary storage has reset. If several public assessments exist, ask which one is the user's before opening one.

Retrieve questions and scoring context through MCP using the assessment's `bank_version`. The canonical source is `assessment/banks/1.0.0.json`. Do not maintain a second questionnaire in skill prose or Rust source. The generated [question bank](references/question-bank.md) and [scoring reference](references/scoring.md) are informational fallbacks; do not infer saved progress from them.

The artifact saves answers. Do not duplicate its writes. For deliberate chat-based edits, read the current assessment, then send only the user's supplied answer with `expected_revision`. `record_answer` returns the saved answer, revision, progress and completion state; use that result instead of immediately fetching the full assessment again. On a revision conflict, reload and reconcile; never silently overwrite another edit.

For completed assessments, `record_answer` and `get_assessment` include results and a handoff summary. Use that exact assessment ID to summarize findings and next actions. Use `calculate_maturity` if a fresh full report is needed. Distinguish completion, implementation score and evidence confidence. Do not invent a final score when overall_index_pct is null. Unknown is not zero; NA requires a reason.

Recommendations include actions, owner roles, effort, dependencies, outcomes and verification evidence. Review template priorities against actual business impact and resources. Do not claim remediation has occurred.

Use `export_assessment` for a versioned JSON backup and `import_assessment` to restore as a new record. Never promise durable storage on Render Free. Do not treat an assessment ID as authentication.

This is an independent ENISA-informed method, not an official ENISA score, certification or endorsement. Never fabricate answers, technical checks or verification. Evidence references and caller-supplied dates remain self-reported. Use fictional/anonymized demo data and do not ask for passwords, API keys or customer data. Match the user's language.

Official context: https://www.enisa.europa.eu/publications/enisa-report-cybersecurity-for-smes
