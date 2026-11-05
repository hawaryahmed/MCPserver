---
name: assessment-overview
description: Use when the user wants to start a cybersecurity maturity assessment, asks what the app does, or requests an overview before answering questions.
---

# Assessment overview

Start by calling `assessment_overview` with `{}` to open the interactive assessment artifact. Do not create an assessment or ask the first assessment question before showing the overview. Summarize the three domains (People, Processes, Technology), the choice of Quick Baseline (12 questions) or Full Assessment (30 questions), the score scale and the final gap/90-day plan. Let the user choose the scope and start in the form.

The form contains actual answer fields; do not replace it with a long chat questionnaire or a static list of blanks. It handles creation after the user clicks Start. Keep surrounding chat short and avoid repeating the artifact.

If `assessment_overview` is not available in this connection, say that the updated server tools need deployment and a connection refresh. Do not claim to have opened an interactive artifact. Offer a short text overview while the update is pending.

Use the connected SME Cyber Maturity tools. This is an independent ENISA-informed self-assessment, not an official ENISA score, audit, certification or endorsement. Never invent answers, evidence, progress or verified controls. Unknown is not zero. Evidence references do not independently verify implementation.

This deployment is an anonymous public demo on Render Free. Use fictional or anonymized data only; do not request secrets or personal/customer data. Explain that saved assessments are shared and temporary when starting. Do not promise durable storage. Match the user's language in the conversation.
