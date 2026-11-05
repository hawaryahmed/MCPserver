use crate::{
    content::Bank,
    model::{Assessment, Score},
};
use serde_json::{json, Value};
pub fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}
pub fn complete(a: &Assessment, b: &Bank) -> bool {
    b.questions(a.mode)
        .iter()
        .all(|q| a.answers.contains_key(&q.id))
}
pub fn status(a: &Assessment, b: &Bank) -> &'static str {
    if complete(a, b) {
        "completed"
    } else {
        "in_progress"
    }
}
pub fn calculate(a: &Assessment, b: &Bank) -> Value {
    let qs = b.questions(a.mode);
    let mut domains = vec![];
    let mut valid_scores = vec![];
    let mut gaps = vec![];
    for domain in &b.domains {
        let (mut sum, mut n, mut unknown, mut na) = (0u32, 0u32, 0u32, 0u32);
        for q in qs.iter().filter(|q| &q.domain == domain) {
            match a.answers.get(&q.id) {
                None => unknown += 1,
                Some(x) if x.value == Score::Unknown => unknown += 1,
                Some(x) if x.value == Score::NotApplicable => na += 1,
                Some(x) => {
                    let score = x.value.numeric().expect("Numeric score");
                    n += 1;
                    sum += score;
                    if score < b.policy.gap_below {
                        let r = &q.remediation;
                        gaps.push(json!({"question_id":q.id,"domain":q.domain,"question":q.text,"observed_score":score,
                            "priority":r.priority,"target":format!("{} days",r.target_days),"target_days":r.target_days,
                            "action":r.action,"owner":x.owner,"owner_role":r.owner_role,"effort":r.effort,"dependencies":r.dependencies,
                            "expected_outcome":r.expected_outcome,"verification":r.verification,"verification_needed":true,
                            "evidence_confidence":"self_reported","has_evidence_reference":x.evidence.is_some(),
                            "evidence_status":if x.evidence.is_some(){"self-reported with evidence reference"}else{"self-reported without evidence"}}));
                    }
                }
            }
        }
        let coverage = if n + unknown > 0 {
            n as f64 * 100.0 / (n + unknown) as f64
        } else {
            100.0
        };
        let score = (n > 0).then(|| sum as f64 * 100.0 / (b.policy.max_score * n) as f64);
        let provisional =
            coverage < b.policy.minimum_coverage_pct || n < b.policy.minimum_numeric_per_domain;
        if !provisional {
            if let Some(s) = score {
                valid_scores.push(s)
            }
        }
        domains.push(json!({"domain":domain,"score_pct":score.map(round1),"coverage_pct":round1(coverage),"numeric":n,"unknown_or_unanswered":unknown,"justified_na":na,"provisional":provisional}));
    }
    let overall = (valid_scores.len() == b.domains.len())
        .then(|| valid_scores.iter().sum::<f64>() / b.domains.len() as f64);
    let maturity = overall.and_then(|s| {
        b.policy
            .maturity_bands
            .iter()
            .rev()
            .find(|band| s >= band.minimum)
            .map(|band| band.label.as_str())
    });
    gaps.sort_by_key(|v| {
        (
            if v["priority"] == "High" { 0 } else { 1 },
            v["observed_score"].as_u64().unwrap_or(9),
        )
    });
    let unknown: Vec<_> = qs
        .iter()
        .filter(|q| {
            a.answers
                .get(&q.id)
                .is_none_or(|a| a.value == Score::Unknown)
        })
        .map(|q| json!({"question_id":q.id,"question":q.text,"status":"unknown"}))
        .collect();
    let with_reference = a.answers.values().filter(|x| x.evidence.is_some()).count();
    json!({"assessment_id":a.id,"organization":a.organization,"mode":a.mode,"bank_version":a.bank_version,"revision":a.revision,"status":status(a,b),
        "official_enisa":false,"assessment_type":"unverified self assessment","domains":domains,"overall_index_pct":overall.map(round1),"plugin_defined_maturity":maturity,
        "gaps":gaps,"needs_verification":unknown,
        "evidence_summary":{"self_reported_answers":a.answers.len(),"answers_with_evidence_reference":with_reference,"independently_verified":0,"confidence":"self_reported"},
        "caveat":"Plugin-defined scoring, not official ENISA grading. A documented evidence reference does not itself verify a control. Quick/full scores are not directly comparable. Priorities are templates requiring business-context validation."})
}
pub fn handoff(a: &Assessment, b: &Bank) -> Value {
    let r = calculate(a, b);
    json!({"assessment_id":a.id,"organization":a.organization,"status":r["status"],"bank_version":a.bank_version,
        "answered":a.answers.len(),"total":b.questions(a.mode).len(),"overall_index_pct":r["overall_index_pct"],"domains":r["domains"],
        "unknown_or_unanswered":r["needs_verification"].as_array().unwrap().len(),"top_gaps":r["gaps"].as_array().unwrap().iter().take(5).cloned().collect::<Vec<_>>(),
        "evidence_confidence":"self_reported","instruction":"Summarize these saved results and practical next actions. Do not infer independently verified controls. Use calculate_maturity with this assessment_id for the full plan."})
}
