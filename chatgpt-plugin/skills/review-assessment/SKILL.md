---
name: review-assessment
description: Use when the user asks to review assessment results, maturity coverage, gaps, or a 30/60/90-day improvement plan.
---

# Results and 90-day plan

Reuse the selected assessment ID. If the completion handoff already supplies that assessment's saved results, summarize those results. Otherwise call `get_assessment`; a completed record includes its report. Call `calculate_maturity` when the full report is absent or a fresh recalculation is requested. Show domain score, coverage, unknowns, justified NA answers and evidence limitations. When the overall index is null, explicitly say it is provisional; do not supply a final maturity label.

Explain that the server's 30/60/90-day priorities are templates. Refine them using the user's actual business impact, exposure, dependencies and available resources. Give each proposed task an owner role, effort, expected outcome and verification evidence. Do not imply changes have been implemented or verified. Recommend affordable native capabilities where suitable.

Use `assessment_overview` with the same assessment ID if the user wants to review or edit answers in the artifact. Do not create a new assessment or overwrite previous answers just to produce a report.

Use the connected SME Cyber Maturity tools. This is an independent ENISA-informed self-assessment, not an official ENISA score, audit, certification or endorsement. Never invent answers, evidence, progress or verified controls. Unknown is not zero. Evidence references do not independently verify implementation.

This deployment is an anonymous public demo on Render Free. Use fictional or anonymized data only; do not request secrets or personal/customer data. Explain that saved assessments are shared and temporary when starting. Do not promise durable storage. Match the user's language in the conversation.

Use the completion handoff if supplied; it names the completed assessment and includes saved results. Distinguish status completed from implemented controls and verified evidence. Report evidence_summary separately from implementation scores. Explain task effort, dependencies, owner role, expected outcome and verification from the authored remediation metadata. Export a versioned backup with export_assessment when requested; restore through import_assessment, which creates a new ID without overwriting the original.
