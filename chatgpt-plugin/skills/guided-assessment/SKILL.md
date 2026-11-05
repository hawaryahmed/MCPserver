---
name: guided-assessment
description: Use when the user wants to answer, continue, resume or edit a cybersecurity maturity questionnaire one question at a time.
---

# Guided assessment

For a new assessment, show `assessment_overview` with `{}` first. For an existing assessment, reuse the assessment ID and call `assessment_overview` with `assessment_id` to reopen the interactive artifact. If the user did not provide an ID and several assessments exist, ask them which one to resume; never silently open another organization's record.

Use the form's one-question-at-a-time flow. Each question includes its ID/domain, a plain-language explanation, an illustrative answer example, a multiline answer field, implementation level, optional evidence reference, owner, and an expandable follow-up question with its own answer field. Keep Back and Save & Next available. Unknown may be saved as U; NA requires a reason. Never copy examples into real answers.

The widget saves answers through `record_answer` and refreshes through `get_assessment`. Do not duplicate those writes in chat or move to the next question on the user's behalf. If the user deliberately answers in chat instead, first read their current assessment, confirm any ambiguous score, then record only their supplied answer and reopen the artifact. Never fill verified_at just because an evidence reference was supplied.

When asked to explain a question further, use `get_question_bank` for the relevant mode and its authored guidance; add a simple contextual explanation without changing the user's answer. Use the assessment ID rather than creating a new record to resume. If a temporary record has expired, explain what happened and ask before starting a replacement. Do not fabricate recovered answers.

Use the connected SME Cyber Maturity tools. This is an independent ENISA-informed self-assessment, not an official ENISA score, audit, certification or endorsement. Never invent answers, evidence, progress or verified controls. Unknown is not zero. Evidence references do not independently verify implementation.

This deployment is an anonymous public demo on Render Free. Use fictional or anonymized data only; do not request secrets or personal/customer data. Explain that saved assessments are shared and temporary when starting. Do not promise durable storage. Match the user's language in the conversation.
