# Original plugin scoring and reporting rubric

## Response scale
0 = absent; 1 = ad hoc; 2 = partially in place; 3 = consistently operated; 4 = measured/tested/improved; U = unknown; NA = not applicable with justification.

## Calculation
For each domain, observed score = sum of numeric answers / (4 * number of numeric answers) * 100. Coverage = number numeric / (numeric + U), excluding justified NA. State counts of numeric, U, and justified NA. When a domain has < 60% coverage or fewer than 3 numeric answers, mark its result *provisional* and do not assign an overall maturity category. Overall index, only when EACH domain meets coverage >= 60% and >= 3 scored items: simple equal-weight mean of the three domain percent scores. Report the score as a descriptive self-assessment, not verified security posture.

Optional INTERNAL maturity descriptors (not ENISA thresholds): 0-24% Initial; 25-49% Developing; 50-74% Managed; 75-100% Improving. Explicitly label as plugin-defined, not official ENISA Foundation/Advanced/Expert. For evidence confidence: low (answers only), moderate (documents supplied and checked for some controls), high only when substantial recent evidence is actually reviewed and/or controls technically verified; never infer evidence confidence from numeric score.

## Prioritization
Use a transparent qualitative priority: Urgent/High/Medium/Low. Consider concrete business impact, internet/external exposure, ease of exploitation, actual known incidents, dependencies, implementation cost/time, and defense-in-depth. Unknown findings are 'needs verification' rather than actual failure. Include why and what evidence would confirm. Raise priority of account recovery, MFA, backups, offboarding, exposed secrets, critical SaaS/identity.

## Deliverables
For each gap provide: domain/question ID, observed answer/evidence status, business risk scenario, proposed control, owner, effort S/M/L, priority, 30/60/90 day target, proof of completion. Avoid invented financial losses or guarantee of risk elimination. Benchmark only if user provides legitimate peer dataset with provenance; otherwise omit.
