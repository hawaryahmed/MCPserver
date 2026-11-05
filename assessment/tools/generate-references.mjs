// Generate human-readable references from the exact bank embedded by Rust.
import { readFile, writeFile } from 'node:fs/promises';
const root=new URL('../../',import.meta.url);
const bank=JSON.parse(await readFile(new URL('assessment/banks/1.0.0.json',root),'utf8'));
const banner=`<!-- GENERATED from assessment/banks/${bank.version}.json. Edit the versioned bank, then regenerate. -->\n`;
let questions=banner+`# Question bank · ${bank.version}\n\n${bank.methodology}\n\n`;
for(const q of bank.questions){
 questions+=`## ${q.id} · ${q.domain}\n\n${q.text}\n\nModes: ${q.modes.join(', ')}.\n\nExplanation: ${q.explanation}\n\nExample (illustrative): ${q.answer_example}\n\nFollow-up for partial/missing implementation: ${q.follow_ups.low}\n\nFollow-up for consistent/tested implementation: ${q.follow_ups.high}\n\nFollow-up for unknown: ${q.follow_ups.unknown}\n\nFollow-up for NA: ${q.follow_ups.na}\n\nRecommended action: ${q.remediation.action}\n\nOwner role: ${q.remediation.owner_role} · Effort: ${q.remediation.effort} · Target: ${q.remediation.target_days} days.\n\nDependencies: ${q.remediation.dependencies.join('; ')}.\n\nOutcome: ${q.remediation.expected_outcome}\n\nVerify with: ${q.remediation.verification}\n\n`;
}
await writeFile(new URL('skills/sme-maturity-assessment/references/question-bank.md',root),questions);
let rubric=banner+`# Scoring policy · ${bank.version}\n\n${bank.methodology}\n\n`;
for(const s of bank.score_scale)rubric+=`- ${s.value}: ${s.label}\n`;
rubric+=`\nCalculate each domain as numeric-score sum / (${bank.policy.max_score} × numeric-answer count) × 100. Exclude NA from the denominator; count Unknown and unanswered as missing coverage, not zero.\n\nCoverage = numeric answers / (numeric answers + unknown/unanswered) × 100. A domain is provisional below ${bank.policy.minimum_coverage_pct}% coverage or fewer than ${bank.policy.minimum_numeric_per_domain} numeric answers. Only report an overall index when every domain passes; average the domain scores equally. Round displayed percentages to one decimal.\n\n`;
for(const b of bank.policy.maturity_bands)rubric+=`- From ${b.minimum}%: ${b.label}\n`;
rubric+=`\nCreate implementation gaps for scores below ${bank.policy.gap_below}. Use each question's authored action, owner role, effort, dependencies, outcome, verification, priority and target. Review template priorities against business context. Completion means every question has an answer; it does not mean every control is implemented or verified.\n\nEvidence confidence is separate: an evidence reference or a caller-supplied verified_at date does not prove independent verification. This demo reports all answers as self-reported. Pin every assessment to its bank_version. Quick and Full results are not directly comparable.\n`;
await writeFile(new URL('skills/sme-maturity-assessment/references/scoring.md',root),rubric);
console.log(`Generated references from bank ${bank.version}`);
