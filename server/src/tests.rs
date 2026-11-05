use crate::{assessment::Service, content, model::*, scoring, storage::Repository};
use serde_json::{json, Value};
use std::path::PathBuf;
struct Fixture {
    service: Service,
    path: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("sme-test-{}", uuid::Uuid::new_v4()));
        Self {
            service: Service::new(Repository::new(path.clone())),
            path,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn call(f: &Fixture, name: &str, args: Value) -> Value {
    f.service.dispatch(name, &args).unwrap()
}
fn create(f: &Fixture) -> Value {
    call(
        f,
        "create_assessment",
        json!({"organization":"Test","mode":"quick"}),
    )
}
fn record(
    f: &Fixture,
    id: &Value,
    q: &str,
    score: &str,
    revision: Option<u64>,
) -> AppResult<Value> {
    let mut args = json!({"assessment_id":id,"question_id":q,"value":score,"answer_text":"Current practice","follow_up_answer":"Follow-up detail"});
    if let Some(v) = revision {
        args["expected_revision"] = json!(v)
    }
    f.service.dispatch("record_answer", &args)
}
#[test]
fn bank_content_is_valid_and_has_legacy_scope() {
    let b = content::latest();
    content::validate(b).unwrap();
    assert_eq!(b.questions(Mode::Quick).len(), 12);
    assert_eq!(b.questions(Mode::Full).len(), 30);
    for q in &b.questions {
        assert!(q.remediation.action.len() > 30);
        assert!(q.remediation.verification.len() > 20);
        assert_ne!(q.follow_ups.low, q.follow_ups.high);
    }
}
#[test]
fn historical_scoring_and_priorities_are_unchanged() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../tests/fixtures/legacy-scoring-v0.4.0.json")).unwrap();
    for case in cases {
        let a: Assessment = serde_json::from_value(case["assessment"].clone()).unwrap();
        assert_eq!(a.bank_version, LEGACY_BANK_VERSION);
        let r = scoring::calculate(&a, content::by_version(&a.bank_version).unwrap());
        let projected = json!({"domains":r["domains"],"overall_index_pct":r["overall_index_pct"],"plugin_defined_maturity":r["plugin_defined_maturity"],
            "gaps":r["gaps"].as_array().unwrap().iter().map(|g|json!({"question_id":g["question_id"],"observed_score":g["observed_score"],"priority":g["priority"],"target":g["target"]})).collect::<Vec<_>>(),
            "unknown_ids":r["needs_verification"].as_array().unwrap().iter().map(|q|q["question_id"].clone()).collect::<Vec<_>>()});
        assert_eq!(projected, case["expected"], "Fixture {}", case["name"]);
    }
}
#[test]
fn legacy_record_loads_and_migrates_without_changing_answers() {
    let f = Fixture::new();
    std::fs::create_dir_all(&f.path).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    std::fs::write(f.path.join(format!("{id}.json")),json!({"id":id,"organization":"Legacy","mode":"quick","answers":{"P01":{"value":"3","evidence":"Ticket-1"}}}).to_string()).unwrap();
    let view = call(&f, "get_assessment", json!({"assessment_id":id}));
    assert_eq!(view["bank_version"], "1.0.0");
    assert_eq!(view["revision"], 0);
    assert_eq!(view["status"], "in_progress");
    assert_eq!(view["answers"]["P01"]["value"], "3");
    let saved = record(&f, &json!(id), "P02", "2", Some(0)).unwrap();
    assert_eq!(saved["revision"], 1);
    let stored = f.service.repository.load(&id).unwrap();
    assert_eq!(stored.bank_version, "1.0.0");
    assert_eq!(stored.answers.len(), 2);
}
#[test]
fn save_returns_delta_and_conflicts_do_not_overwrite() {
    let f = Fixture::new();
    let a = create(&f);
    let id = &a["assessment_id"];
    let saved = record(&f, id, "P01", "3", Some(0)).unwrap();
    assert_eq!(saved["answer"]["answer_text"], "Current practice");
    assert_eq!(saved["answered"], 1);
    assert_eq!(saved["revision"], 1);
    assert!(saved.get("questions").is_none());
    assert!(matches!(
        record(&f, id, "P01", "0", Some(0)),
        Err(AppError::Conflict {
            current_revision: 1
        })
    ));
    assert_eq!(
        call(&f, "get_assessment", json!({"assessment_id":id}))["answers"]["P01"]["value"],
        "3"
    );
}
#[test]
fn completion_and_chat_handoff_include_saved_results() {
    let f = Fixture::new();
    let a = create(&f);
    let id = &a["assessment_id"];
    let mut result = Value::Null;
    for (rev, q) in content::latest().questions(Mode::Quick).iter().enumerate() {
        result = record(&f, id, &q.id, "3", Some(rev as u64)).unwrap()
    }
    assert_eq!(result["status"], "completed");
    assert_eq!(result["report"]["overall_index_pct"], 75.0);
    assert_eq!(result["handoff"]["assessment_id"], *id);
    assert_eq!(result["handoff"]["status"], "completed");
    assert_eq!(result["handoff"]["overall_index_pct"], 75.0);
    assert_eq!(
        call(&f, "get_assessment", json!({"assessment_id":id}))["report"]["overall_index_pct"],
        75.0
    );
    let updated = record(&f, id, "P01", "1", Some(12)).unwrap();
    assert_eq!(updated["status"], "completed");
    assert!(updated["report"]["overall_index_pct"].as_f64().unwrap() < 75.0);
}
#[test]
fn validation_is_enforced_on_server() {
    let f = Fixture::new();
    let a = create(&f);
    let id = &a["assessment_id"];
    for args in [
        json!({"assessment_id":id,"question_id":"P01","value":"5"}),
        json!({"assessment_id":id,"question_id":"P01","value":3}),
        json!({"assessment_id":id,"question_id":"P01","value":"NA"}),
        json!({"assessment_id":id,"question_id":"P01","value":"3","answer_text":"x".repeat(12001)}),
        json!({"assessment_id":id,"question_id":"P01","value":"3","unrecognized":true}),
    ] {
        assert!(f.service.dispatch("record_answer", &args).is_err())
    }
    assert_eq!(
        call(&f, "get_assessment", json!({"assessment_id":id}))["answered"],
        0
    );
}
#[test]
fn export_restore_uses_new_id_preserves_version_and_does_not_overwrite() {
    let f = Fixture::new();
    let a = create(&f);
    let id = &a["assessment_id"];
    record(&f, id, "P01", "2", Some(0)).unwrap();
    let backup = call(&f, "export_assessment", json!({"assessment_id":id}));
    let restored = call(&f, "import_assessment", json!({"backup":backup["backup"]}));
    assert_ne!(restored["assessment_id"], *id);
    assert_eq!(restored["revision"], 0);
    assert_eq!(restored["bank_version"], "1.0.0");
    assert_eq!(restored["answers"]["P01"]["value"], "2");
    assert_eq!(
        call(&f, "list_assessments", json!({}))["assessments"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let mut invalid = backup["backup"].clone();
    invalid["assessment"]["bank_version"] = json!("99.0.0");
    assert!(f
        .service
        .dispatch("import_assessment", &json!({"backup":invalid}))
        .is_err());
    let mut invalid = backup["backup"].clone();
    invalid["assessment"]["answers"]["P01"]["value"] = json!("NA");
    assert!(f
        .service
        .dispatch("import_assessment", &json!({"backup":invalid}))
        .is_err());
}
#[test]
fn evidence_references_and_claimed_dates_do_not_verify_controls() {
    let f = Fixture::new();
    let a = create(&f);
    let id = &a["assessment_id"];
    call(
        &f,
        "record_answer",
        json!({"assessment_id":id,"question_id":"P01","value":"1","evidence":"Policy","verified_at":"2026-11-05"}),
    );
    let r = call(&f, "calculate_maturity", json!({"assessment_id":id}));
    assert_eq!(r["evidence_summary"]["independently_verified"], 0);
    assert_eq!(r["evidence_summary"]["confidence"], "self_reported");
    assert_eq!(r["gaps"][0]["owner_role"], "Founder");
    assert!(!r["gaps"][0]["dependencies"].as_array().unwrap().is_empty());
}
#[test]
fn mcp_resources_and_action_metadata() {
    assert!(
        crate::ui::read(crate::ui::URI).unwrap()["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Assessment overview")
    );
    assert!(crate::ui::read("ui://unknown").is_err());
    for t in crate::mcp::tools() {
        let name = t["name"].as_str().unwrap();
        let write = [
            "create_assessment",
            "record_answer",
            "delete_assessment",
            "import_assessment",
        ]
        .contains(&name);
        assert_eq!(t["annotations"]["readOnlyHint"], !write);
        if name == "assessment_overview" {
            assert_eq!(t["_meta"]["ui"]["resourceUri"], crate::ui::URI)
        } else {
            assert!(t["_meta"]["ui"].get("resourceUri").is_none())
        }
    }
}

#[test]
fn legacy_bank_question_wording_and_guidance_are_frozen() {
    let old: Vec<Value> = serde_json::from_str(include_str!(
        "../tests/fixtures/legacy-questions-v0.4.0.json"
    ))
    .unwrap();
    let bank = content::by_version(LEGACY_BANK_VERSION).unwrap();
    for q in old {
        let current = bank.question_json(
            bank.question(Mode::Full, q["id"].as_str().unwrap())
                .unwrap(),
        );
        for key in [
            "id",
            "domain",
            "text",
            "explanation",
            "answer_example",
            "follow_up",
        ] {
            assert_eq!(
                current[key], q[key],
                "Change requires a new bank version: {} {key}",
                q["id"]
            );
        }
    }
}

#[test]
fn registered_bank_version_must_match_its_content() {
    let raw = include_str!("../../assessment/banks/1.0.0.json");
    assert!(content::parse_bank("2.0.0", raw).is_err());
    assert_eq!(
        content::by_version(LEGACY_BANK_VERSION).unwrap().version,
        LEGACY_BANK_VERSION
    );
}
