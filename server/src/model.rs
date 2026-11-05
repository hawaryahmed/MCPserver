use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const LEGACY_BANK_VERSION: &str = "1.0.0";
fn legacy_bank() -> String {
    LEGACY_BANK_VERSION.into()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Quick,
    Full,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Score {
    #[serde(rename = "0")]
    Absent,
    #[serde(rename = "1")]
    AdHoc,
    #[serde(rename = "2")]
    Partial,
    #[serde(rename = "3")]
    Consistent,
    #[serde(rename = "4")]
    Improving,
    #[serde(rename = "U")]
    Unknown,
    #[serde(rename = "NA")]
    NotApplicable,
}
impl Score {
    pub fn numeric(self) -> Option<u32> {
        match self {
            Self::Absent => Some(0),
            Self::AdHoc => Some(1),
            Self::Partial => Some(2),
            Self::Consistent => Some(3),
            Self::Improving => Some(4),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    pub value: Score,
    #[serde(default)]
    pub answer_text: Option<String>,
    #[serde(default)]
    pub follow_up_answer: Option<String>,
    #[serde(default)]
    pub evidence: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub na_justification: Option<String>,
    // Retain legacy caller-supplied dates without treating them as verified evidence.
    #[serde(default)]
    pub verified_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assessment {
    pub id: String,
    pub organization: String,
    pub mode: Mode,
    #[serde(default = "legacy_bank")]
    pub bank_version: String,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub answers: BTreeMap<String, Answer>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Empty {}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OverviewInput {
    pub assessment_id: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssessmentInput {
    pub assessment_id: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestionBankInput {
    #[serde(default = "full")]
    pub mode: Mode,
    pub bank_version: Option<String>,
}
fn full() -> Mode {
    Mode::Full
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateInput {
    pub organization: String,
    #[serde(default)]
    pub mode: Mode,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub assessment_id: String,
    pub question_id: String,
    pub value: Score,
    pub answer_text: Option<String>,
    pub follow_up_answer: Option<String>,
    pub evidence: Option<String>,
    pub owner: Option<String>,
    pub na_justification: Option<String>,
    pub verified_at: Option<String>,
    pub expected_revision: Option<u64>,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Backup {
    pub format: String,
    pub format_version: u32,
    pub assessment: Assessment,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportInput {
    pub backup: Backup,
}

#[derive(Debug)]
pub enum AppError {
    Validation(String),
    NotFound,
    Conflict { current_revision: u64 },
    Storage,
}
impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "invalid_input",
            Self::NotFound => "not_found",
            Self::Conflict { .. } => "revision_conflict",
            Self::Storage => "storage_error",
        }
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
        Self::Validation(s)=>write!(f,"{s}"), Self::NotFound=>write!(f,"Assessment not found. Demo storage may have reset."),
        Self::Conflict{current_revision}=>write!(f,"Another form changed this assessment (revision {current_revision}). Reload before saving; your draft has not been applied."),
        Self::Storage=>write!(f,"Assessment storage is unavailable. Please retry."),
    }
    }
}
pub type AppResult<T> = Result<T, AppError>;
pub fn parse<T: for<'de> Deserialize<'de>>(v: &serde_json::Value) -> AppResult<T> {
    serde_json::from_value(v.clone()).map_err(|e| AppError::Validation(e.to_string()))
}
pub fn text(value: Option<String>, max: usize, field: &str) -> AppResult<Option<String>> {
    match value {
        Some(s) => {
            let s = s.trim().to_owned();
            if s.chars().count() > max {
                return Err(AppError::Validation(format!(
                    "{field} must be at most {max} characters"
                )));
            }
            Ok((!s.is_empty()).then_some(s))
        }
        None => Ok(None),
    }
}
pub fn validate_answer(a: &Answer) -> AppResult<()> {
    for (value, max, field) in [
        (&a.answer_text, 12000, "answer_text"),
        (&a.follow_up_answer, 6000, "follow_up_answer"),
        (&a.evidence, 4000, "evidence"),
        (&a.owner, 160, "owner"),
        (&a.na_justification, 4000, "na_justification"),
        (&a.verified_at, 80, "verified_at"),
    ] {
        text(value.clone(), max, field)?;
    }
    validate_applicability(a)
}
// Existing records predate text limits. Read them without silently changing/rejecting their text.
pub fn validate_applicability(a: &Answer) -> AppResult<()> {
    if a.value == Score::NotApplicable
        && a.na_justification
            .as_ref()
            .is_none_or(|s| s.trim().is_empty())
    {
        return Err(AppError::Validation("NA requires na_justification".into()));
    }
    Ok(())
}
