use crate::{
    content::{self, Bank},
    model::*,
    scoring,
    storage::Repository,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub struct Service {
    pub repository: Repository,
}
impl Service {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
    pub fn validate(&self, a: &Assessment) -> AppResult<&'static Bank> {
        self.repository.path(&a.id)?;
        if a.organization.trim().is_empty() || a.organization.chars().count() > 160 {
            return Err(AppError::Validation(
                "organization must be 1–160 characters".into(),
            ));
        }
        let b = content::by_version(&a.bank_version)?;
        for (id, answer) in &a.answers {
            b.question(a.mode, id)?;
            validate_applicability(answer)?;
        }
        Ok(b)
    }
    fn load(&self, id: &str) -> AppResult<Assessment> {
        let a = self.repository.load(id)?;
        self.validate(&a)?;
        Ok(a)
    }
    pub fn snapshot(&self, a: &Assessment) -> AppResult<Value> {
        let b = self.validate(a)?;
        let qs = b.questions(a.mode);
        let next = qs.iter().position(|q| !a.answers.contains_key(&q.id));
        let done = scoring::complete(a, b);
        Ok(
            json!({"view":"questionnaire","assessment_id":a.id,"organization":a.organization,"mode":a.mode,"bank_version":a.bank_version,
            "revision":a.revision,"status":scoring::status(a,b),"answers":a.answers,
            "questions":qs.iter().map(|q|b.question_json(q)).collect::<Vec<_>>(),"next_index":next,"answered":a.answers.len(),"total":qs.len(),
            "score_scale":b.score_scale,"demo_notice":crate::ui::DEMO_NOTICE,
            "report":if done{Some(scoring::calculate(a,b))}else{None},"handoff":if done{Some(scoring::handoff(a,b))}else{None}}),
        )
    }
    pub fn get(&self, input: AssessmentInput) -> AppResult<Value> {
        self.snapshot(&self.load(&input.assessment_id)?)
    }
    pub fn create(&self, input: CreateInput) -> AppResult<Value> {
        let organization = input.organization.trim().to_owned();
        let a = Assessment {
            id: Uuid::new_v4().to_string(),
            organization,
            mode: input.mode,
            bank_version: content::latest().version.clone(),
            revision: 0,
            answers: Default::default(),
        };
        self.validate(&a)?;
        self.repository.save(&a)?;
        let mut result = self.snapshot(&a)?;
        result["questions_total"] = result["total"].clone();
        Ok(result)
    }
    pub fn record(&self, input: RecordInput) -> AppResult<Value> {
        let mut a = self.load(&input.assessment_id)?;
        if input
            .expected_revision
            .is_some_and(|revision| revision != a.revision)
        {
            return Err(AppError::Conflict {
                current_revision: a.revision,
            });
        }
        let id = input.question_id.to_uppercase();
        let b = content::by_version(&a.bank_version)?;
        b.question(a.mode, &id)?;
        let answer = Answer {
            value: input.value,
            answer_text: text(input.answer_text, 12000, "answer_text")?,
            follow_up_answer: text(input.follow_up_answer, 6000, "follow_up_answer")?,
            evidence: text(input.evidence, 4000, "evidence")?,
            owner: text(input.owner, 160, "owner")?,
            na_justification: text(input.na_justification, 4000, "na_justification")?,
            verified_at: text(input.verified_at, 80, "verified_at")?,
        };
        validate_answer(&answer)?;
        a.answers.insert(id.clone(), answer.clone());
        a.revision = a
            .revision
            .checked_add(1)
            .ok_or_else(|| AppError::Validation("Revision limit reached".into()))?;
        self.repository.save(&a)?;
        let done = scoring::complete(&a, b);
        let qs = b.questions(a.mode);
        let next = qs.iter().position(|q| !a.answers.contains_key(&q.id));
        Ok(
            json!({"assessment_id":a.id,"question_id":id,"saved":true,"answered":a.answers.len(),"total":qs.len(),"revision":a.revision,"status":scoring::status(&a,b),"bank_version":a.bank_version,
            "answer":answer,"next_index":next,"evidence_confidence":"self_reported",
            "report":if done{Some(scoring::calculate(&a,b))}else{None},"handoff":if done{Some(scoring::handoff(&a,b))}else{None}}),
        )
    }
    pub fn calculate(&self, input: AssessmentInput) -> AppResult<Value> {
        let a = self.load(&input.assessment_id)?;
        Ok(scoring::calculate(
            &a,
            content::by_version(&a.bank_version)?,
        ))
    }
    pub fn list(&self) -> AppResult<Value> {
        let records = self.repository.list()?;
        let mut items = vec![];
        for a in records {
            let b = self.validate(&a)?;
            items.push(json!({"assessment_id":a.id,"organization":a.organization,"mode":a.mode,"answered":a.answers.len(),"total":b.questions(a.mode).len(),"bank_version":a.bank_version,"revision":a.revision,"status":scoring::status(&a,b)}))
        }
        Ok(json!({"assessments":items}))
    }
    pub fn delete(&self, input: AssessmentInput) -> AppResult<Value> {
        self.repository.delete(&input.assessment_id)?;
        Ok(json!({"deleted":true}))
    }
    pub fn export(&self, input: AssessmentInput) -> AppResult<Value> {
        let a = self.load(&input.assessment_id)?;
        let filename = format!("sme-assessment-{}.json", a.id);
        let backup = Backup {
            format: "sme-maturity-assessment".into(),
            format_version: 1,
            assessment: a,
        };
        Ok(json!({"filename":filename,"mime_type":"application/json","backup":backup}))
    }
    pub fn import(&self, input: ImportInput) -> AppResult<Value> {
        if input.backup.format != "sme-maturity-assessment" || input.backup.format_version != 1 {
            return Err(AppError::Validation(
                "Unsupported backup format or version".into(),
            ));
        }
        let mut a = input.backup.assessment;
        for answer in a.answers.values() {
            validate_answer(answer)?;
        }
        self.validate(&a)?;
        // Always create a new record, never overwrite an existing assessment on import.
        a.id = Uuid::new_v4().to_string();
        a.revision = 0;
        self.repository.save(&a)?;
        let mut r = self.snapshot(&a)?;
        r["imported"] = json!(true);
        Ok(r)
    }
    pub fn question_bank(&self, input: QuestionBankInput) -> AppResult<Value> {
        let b = match input.bank_version {
            Some(ref v) => content::by_version(v)?,
            None => content::latest(),
        };
        Ok(
            json!({"mode":input.mode,"bank_version":b.version,"official_enisa":false,"score_scale":b.scale_map(),"questions":b.questions(input.mode).iter().map(|q|b.question_json(q)).collect::<Vec<_>>()}),
        )
    }
    pub fn dispatch(&self, name: &str, args: &Value) -> AppResult<Value> {
        match name {
            "assessment_overview" => {
                let i: OverviewInput = parse(args)?;
                match i.assessment_id {
                    Some(id) => self.get(AssessmentInput { assessment_id: id }),
                    None => Ok(crate::ui::overview(content::latest())),
                }
            }
            "get_assessment" => self.get(parse(args)?),
            "get_question_bank" => self.question_bank(parse(args)?),
            "create_assessment" => self.create(parse(args)?),
            "record_answer" => self.record(parse(args)?),
            "calculate_maturity" => self.calculate(parse(args)?),
            "list_assessments" => {
                let _: Empty = parse(args)?;
                self.list()
            }
            "delete_assessment" => self.delete(parse(args)?),
            "export_assessment" => self.export(parse(args)?),
            "import_assessment" => self.import(parse(args)?),
            _ => Err(AppError::Validation(format!("Unknown tool: {name}"))),
        }
    }
}
