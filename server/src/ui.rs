use crate::{
    content::Bank,
    model::{AppError, AppResult, Mode},
};
use serde_json::{json, Value};
pub const URI: &str = "ui://sme-maturity/assessment-v3.html";
pub const HTML: &str = include_str!("../../web/dist/assessment.html");
pub const DEMO_NOTICE:&str="Public demo: everyone can access saved assessments. Use fictional data. Render Free storage is temporary.";
pub fn overview(b: &Bank) -> Value {
    json!({"view":"overview","title":"Cybersecurity maturity assessment","bank_version":b.version,"domains":b.domains,"score_scale":b.score_scale,
        "modes":[{"id":"quick","label":"Quick baseline","questions":b.questions(Mode::Quick).len()},{"id":"full","label":"Full assessment","questions":b.questions(Mode::Full).len()}],
        "steps":["Choose your scope","Answer one question at a time","Review gaps and a 30/60/90-day plan"],"disclaimer":b.methodology,"sources":b.sources,"demo_notice":DEMO_NOTICE})
}
pub fn resource() -> Value {
    json!({"uri":URI,"name":"Cyber maturity questionnaire","mimeType":"text/html;profile=mcp-app","description":"Assessment overview and one-question-at-a-time form"})
}
pub fn read(uri: &str) -> AppResult<Value> {
    if uri != URI {
        return Err(AppError::Validation("Unknown UI resource".into()));
    }
    Ok(
        json!({"contents":[{"uri":URI,"mimeType":"text/html;profile=mcp-app","text":HTML,"_meta":{"ui":{"prefersBorder":true,"csp":{"connectDomains":[],"resourceDomains":[]}},"openai/widgetDescription":"Assessment overview, one-question form, saved results and backup/restore."}}]}),
    )
}
