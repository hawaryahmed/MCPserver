use crate::model::{AppError, AppResult, Mode, Score};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub version: String,
    pub methodology: String,
    pub sources: Vec<Source>,
    pub domains: Vec<String>,
    pub score_scale: Vec<Scale>,
    pub policy: Policy,
    pub questions: Vec<Question>,
}
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub title: String,
    pub url: String,
}
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scale {
    pub value: Score,
    pub label: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub max_score: u32,
    pub minimum_coverage_pct: f64,
    pub minimum_numeric_per_domain: u32,
    pub gap_below: u32,
    pub maturity_bands: Vec<Band>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    pub minimum: f64,
    pub label: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    pub domain: String,
    pub modes: Vec<Mode>,
    pub text: String,
    pub explanation: String,
    pub answer_example: String,
    pub follow_ups: FollowUps,
    pub remediation: Remediation,
}
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct FollowUps {
    pub low: String,
    pub high: String,
    pub unknown: String,
    pub na: String,
}
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Remediation {
    pub action: String,
    pub owner_role: String,
    pub effort: String,
    pub dependencies: Vec<String>,
    pub expected_outcome: String,
    pub verification: String,
    pub priority: String,
    pub target_days: u32,
}
const V1_JSON: &str = include_str!("../../assessment/banks/1.0.0.json");
static V1: OnceLock<Bank> = OnceLock::new();
pub fn latest() -> &'static Bank {
    by_version("1.0.0").expect("Bundled bank must be available")
}
pub fn by_version(version: &str) -> AppResult<&'static Bank> {
    match version {"1.0.0"=>Ok(V1.get_or_init(||{let b:Bank=serde_json::from_str(V1_JSON).expect("Invalid bundled bank");validate(&b).expect("Invalid bank configuration");b})),_=>Err(AppError::Validation(format!("Unsupported question bank version: {version}. This assessment cannot be silently rescored.")))}
}
pub fn validate(b: &Bank) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    if b.domains
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != b.domains.len()
    {
        return Err("Duplicate domain".into());
    }

    if b.version.is_empty()
        || b.domains.is_empty()
        || b.policy.max_score != 4
        || b.policy.minimum_numeric_per_domain == 0
        || !(0.0..=100.0).contains(&b.policy.minimum_coverage_pct)
        || b.policy.gap_below > b.policy.max_score
    {
        return Err("Invalid scoring policy".into());
    }
    if b.policy.maturity_bands.is_empty()
        || b.policy.maturity_bands[0].minimum != 0.0
        || b.policy
            .maturity_bands
            .windows(2)
            .any(|w| w[0].minimum >= w[1].minimum)
    {
        return Err("Invalid maturity bands".into());
    }
    let all = [
        Score::Absent,
        Score::AdHoc,
        Score::Partial,
        Score::Consistent,
        Score::Improving,
        Score::Unknown,
        Score::NotApplicable,
    ];
    if b.score_scale.len() != all.len()
        || all.iter().any(|v| {
            b.score_scale
                .iter()
                .filter(|s| s.value == *v && !s.label.is_empty())
                .count()
                != 1
        })
    {
        return Err("Incomplete score scale".into());
    }
    for q in &b.questions {
        if !q
            .id
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
            || !ids.insert(&q.id)
            || q.id.is_empty()
            || !b.domains.contains(&q.domain)
            || q.modes.is_empty()
            || q.text.is_empty()
            || q.explanation.is_empty()
            || q.answer_example.is_empty()
            || q.remediation.action.is_empty()
            || q.remediation.verification.is_empty()
            || q.remediation.dependencies.is_empty()
            || ![30, 60, 90].contains(&q.remediation.target_days)
        {
            return Err(format!("Invalid question {}", q.id));
        }
    }
    Ok(())
}
impl Bank {
    pub fn questions(&self, mode: Mode) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| q.modes.contains(&mode))
            .collect()
    }
    pub fn question(&self, mode: Mode, id: &str) -> AppResult<&Question> {
        self.questions(mode)
            .into_iter()
            .find(|q| q.id == id)
            .ok_or_else(|| {
                AppError::Validation(format!("Question {id} is not in this assessment mode"))
            })
    }
    pub fn question_json(&self, q: &Question) -> Value {
        json!({"id":q.id,"domain":q.domain,"text":q.text,"explanation":q.explanation,"answer_example":q.answer_example,"follow_up":q.follow_ups.high,"follow_ups":q.follow_ups,"evidence_hint":"Use a non-sensitive document name, ticket reference, review date or anonymized description. Do not paste passwords, keys or customer data."})
    }
    pub fn scale_map(&self) -> Value {
        let mut out = serde_json::Map::new();
        for s in &self.score_scale {
            out.insert(
                serde_json::to_value(s.value)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .into(),
                json!(s.label),
            );
        }
        Value::Object(out)
    }
}
