use crate::{assessment::Service, model::*, ui};
use schemars::{schema_for, JsonSchema};
use serde_json::{json, Value};
fn tool<T: JsonSchema>(
    name: &str,
    title: &str,
    description: &str,
    write: bool,
    destructive: bool,
) -> Value {
    let mut schema = serde_json::to_value(schema_for!(T)).expect("Input schema");
    schema.as_object_mut().unwrap().remove("$schema");
    json!({"name":name,"title":title,"description":description,"inputSchema":schema,"securitySchemes":[{"type":"noauth"}],
        "annotations":{"title":title,"readOnlyHint":!write,"destructiveHint":destructive,"idempotentHint":!write,"openWorldHint":false},
        "_meta":{"ui":{"visibility":["model","app"]},"openai/widgetAccessible":true,"securitySchemes":[{"type":"noauth"}]}})
}
pub fn tools() -> Vec<Value> {
    let mut list=vec![
        tool::<OverviewInput>("assessment_overview","Assessment overview","Start here to open the assessment overview artifact, then answer one question at a time. Pass assessment_id to resume; completed assessments open their saved results.",false,false),
        tool::<AssessmentInput>("get_assessment","View assessment","Read the saved assessment, pinned questionnaire version, revision, completion state and question guidance.",false,false),
        tool::<QuestionBankInput>("get_question_bank","View questions and guidance","Read the versioned questionnaire and shared score scale. Use an assessment's bank_version when reviewing its questions.",false,false),
        tool::<CreateInput>("create_assessment","Start assessment","Create an assessment and return its initial form data, pinned bank version and revision. Public demo storage is temporary.",true,false),
        tool::<RecordInput>("record_answer","Save or update answer","Save one answer and return the saved answer, progress and revision in one call. Send expected_revision to reject stale updates. The last answer returns completed results and a chat handoff; no extra get_assessment call is needed.",true,false),
        tool::<AssessmentInput>("calculate_maturity","View results and 90-day plan","Read implementation scores, evidence confidence, coverage and actionable 30/60/90-day starting tasks using the assessment's pinned bank version.",false,false),
        tool::<Empty>("list_assessments","List assessments","List public demo assessments with progress, completion state, bank version and revision.",false,false),
        tool::<AssessmentInput>("delete_assessment","Delete assessment","Delete one assessment from temporary demo storage.",true,true),
        tool::<AssessmentInput>("export_assessment","Export assessment backup","Read a versioned JSON backup of an assessment for download and later restoration. Export does not change the record.",false,false),
        tool::<ImportInput>("import_assessment","Restore assessment backup","Validate an exported backup and restore it as a new assessment ID. Never overwrites an existing record. Unknown bank versions are rejected.",true,false),
    ];
    let render = &mut list[0];
    render["_meta"]["ui"]["resourceUri"] = json!(ui::URI);
    render["_meta"]["openai/outputTemplate"] = json!(ui::URI);
    render["_meta"]["openai/toolInvocation/invoking"] = json!("Opening assessment…");
    render["_meta"]["openai/toolInvocation/invoked"] = json!("Assessment ready");
    render["_meta"]["openai/ui"] =
        json!({"entrypoints":[{"type":"thread"}],"availableDisplayModes":["inline","fullscreen"]});
    list
}
pub fn response(id: Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}
pub fn error(id: Value, code: i32, message: impl ToString) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message.to_string()}})
}
pub fn handle(req: &Value, service: &Service) -> Option<Value> {
    let id = req.get("id")?.clone();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    Some(match method {
        "initialize" => response(
            id,
            json!({"protocolVersion":req.pointer("/params/protocolVersion").and_then(Value::as_str).filter(|v|crate::SUPPORTED_VERSIONS.contains(v)).unwrap_or("2025-11-25"),"capabilities":{"tools":{"listChanged":false},"resources":{}},"serverInfo":{"name":"enisa-sme-mcp","version":env!("CARGO_PKG_VERSION")}}),
        ),
        "ping" => response(id, json!({})),
        "resources/list" => response(id, json!({"resources":[ui::resource()]})),
        "resources/templates/list" => response(id, json!({"resourceTemplates":[]})),
        "resources/read" => match ui::read(
            req.pointer("/params/uri")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ) {
            Ok(v) => response(id, v),
            Err(e) => error(id, -32602, e),
        },
        "tools/list" => response(id, json!({"tools":tools()})),
        "tools/call" => {
            let name = req
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            let args = req
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match service.dispatch(name, &args) {
                Ok(v) => response(
                    id,
                    json!({"content":[{"type":"text","text":v.to_string()}],"structuredContent":v,"isError":false}),
                ),
                Err(e) => response(
                    id,
                    json!({"content":[{"type":"text","text":e.to_string()}],"structuredContent":{"error":{"code":e.code(),"message":e.to_string()}},"isError":true}),
                ),
            }
        }
        _ => error(id, -32601, format!("Method not found: {method}")),
    })
}
