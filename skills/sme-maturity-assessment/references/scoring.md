<!-- GENERATED from assessment/banks/1.0.0.json. Edit the versioned bank, then regenerate. -->
# Scoring policy · 1.0.0

Independent ENISA-informed questionnaire; not official ENISA grading.

- 0: Not in place
- 1: Ad hoc
- 2: Partly implemented
- 3: Consistently implemented
- 4: Measured, tested and improved
- U: Unknown
- NA: Not applicable

Calculate each domain as numeric-score sum / (4 × numeric-answer count) × 100. Exclude NA from the denominator; count Unknown and unanswered as missing coverage, not zero.

Coverage = numeric answers / (numeric answers + unknown/unanswered) × 100. A domain is provisional below 60% coverage or fewer than 3 numeric answers. Only report an overall index when every domain passes; average the domain scores equally. Round displayed percentages to one decimal.

- From 0%: Initial
- From 25%: Developing
- From 50%: Managed
- From 75%: Improving

Create implementation gaps for scores below 3. Use each question's authored action, owner role, effort, dependencies, outcome, verification, priority and target. Review template priorities against business context. Completion means every question has an answer; it does not mean every control is implemented or verified.

Evidence confidence is separate: an evidence reference or a caller-supplied verified_at date does not prove independent verification. This demo reports all answers as self-reported. Pin every assessment to its bank_version. Quick and Full results are not directly comparable.
