use serde_json::{json, Value};

pub const URI: &str = "ui://sme-maturity/assessment-v1.html";
pub const HTML: &str = include_str!("../../web/dist/assessment.html");

// Original, ENISA-informed guidance; examples are illustrative, never answers.
const HELP: [(&str, &str, &str); 30] = [
("Name a person who owns security decisions and can approve time or budget.", "Our founder owns security; the IT provider handles changes after approval.", "Who approves an urgent security fix when the usual owner is away?"),
("Include employees and contractors. Look for practical onboarding, not just a policy link.", "New starters get phishing examples and an MFA walkthrough; contractors are not yet included.", "When was the latest starter trained, and where is completion recorded?"),
("People should know a clear, quick way to report suspicious activity.", "We use a dedicated email and chat channel monitored by the operations lead.", "Who monitors reports outside normal working hours?"),
("Check whether people handling money or admin access independently verify unusual requests.", "Bank-detail changes require a callback to a known number and a second approver.", "How would you verify an urgent payment request apparently from the founder?"),
("Remove or adjust contractor access when roles or engagements change.", "The project owner checks accounts at contract end; SaaS access is still manual.", "Which accounts did the most recent departing contractor lose access to?"),
("Write down what your provider secures, what you secure, and how issues are escalated.", "Our IT agreement lists patching and backup responsibilities but not incident reporting.", "Who is responsible if the provider detects an account compromise?"),
("Include security in leadership decisions about products, vendors and spending.", "Leadership reviews major risks monthly and assigns a named owner for decisions.", "What was the most recent security decision leadership made?"),
("Rehearse a realistic incident so people know their roles before an emergency.", "We rehearsed a stolen admin account scenario and logged actions to improve.", "Which improvement from the latest exercise has been completed?"),
("People should know which data is sensitive and how to share, store and dispose of it.", "Staff use approved storage and avoid customer data in personal chat or AI tools.", "How are staff told which data can be shared with external tools?"),
("Use incidents and observed weaknesses to improve training, then check understanding.", "After a phishing report, we updated onboarding and tested the new reporting steps.", "How did you check whether the change improved staff behavior?"),
("List services that matter to the business, their systems, data and accountable owners.", "A shared inventory maps our app, identity provider and customer database to owners.", "When was the inventory last reviewed and who updates it?"),
("Rank specific risks using business impact and likelihood, and revisit them regularly.", "We review a risk register each quarter; customer-data loss and account takeover rank highest.", "What changed since the last risk review, and did priorities change?"),
("Track security and privacy obligations from contracts and applicable requirements.", "Customer contracts and privacy obligations are tracked with owners and review dates.", "Who checks a new customer contract for security commitments?"),
("Keep usable incident contacts and initial steps for containing likely incidents.", "Our runbook lists the incident lead, provider contacts and account lockout steps.", "Could the team access the runbook if company email were unavailable?"),
("Document access approvals and reliably remove access when people leave.", "The manager approves access by ticket; offboarding includes identity, Git and SaaS accounts.", "How was the latest offboarding checked for missed accounts?"),
("Define which services need recovery, how quickly, and how much data loss is tolerable.", "The app must recover within a day with no more than a day's data loss; the plan names an owner.", "Have recovery targets been tested against a real restore?"),
("Check critical vendors before use and when their access or risk changes.", "We review security information and data access for critical vendors annually.", "Which vendor failure would cause the biggest business interruption?"),
("Give findings an owner and due date; record risk acceptance rather than silently ignoring them.", "We track patches and vulnerabilities with deadlines; exceptions need founder approval.", "What happens when a high-priority finding passes its deadline?"),
("Review and test security-impacting changes before deploying them and retain a record.", "Cloud permissions and production changes require review and a rollback plan.", "How could you trace the last production permission change?"),
("Test recovery and incident plans, record lessons, and follow through on improvements.", "A recovery exercise found a missing secret; the runbook and recovery process were corrected.", "Who verifies that exercise actions are closed?"),
("Prioritize admin, email and financial accounts. Passkeys or security keys resist phishing where supported.", "Admin accounts use security keys; other critical accounts use MFA; one vendor is an exception.", "Which critical accounts are still exceptions, and who owns fixing them?"),
("Use supported operating systems, apply updates and check protection is functioning.", "Company laptops are supported, centrally updated and monitored for protection failures.", "How do you find devices that missed updates or lost protection?"),
("A backup is only useful if you can restore it. Include essential settings and dependencies.", "Daily backups are separate from production; a sample restore was tested last month.", "What was restored, how long did it take, and did you verify its integrity?"),
("Use individual accounts and only the permissions people actually need.", "Staff have unique accounts; admin access is limited and reviewed quarterly.", "Which users currently have privileged access, and why?"),
("Track cloud accounts, SaaS, devices and external services, including their owners.", "Our inventory includes approved SaaS and devices, but personal subscriptions need review.", "How would you discover a tool bought outside the normal approval process?"),
("Check how sensitive data is protected while moving and while stored, including backups.", "Customer traffic uses TLS; databases and backups are encrypted with managed keys.", "Who can access encryption keys or decrypt sensitive data?"),
("Review critical configurations rather than assuming vendor defaults are always sufficient.", "We check cloud storage, sharing and identity settings against a documented baseline.", "When did the last configuration review occur and what changed?"),
("Collect useful logs and assign someone to act on meaningful security alerts.", "Admin sign-ins and cloud changes are logged; suspicious login alerts go to the on-call owner.", "How do you know alerts are reaching someone and being acted on?"),
("Find exposed systems and vulnerable dependencies, then remediate according to risk.", "Our pipeline checks dependencies weekly; internet-facing findings get assigned deadlines.", "Which unresolved public-facing finding has the highest business impact?"),
("Use managed secret stores and rotate exposed secrets; avoid code, chat and shared files.", "Production credentials are in a secret manager with scoped access and rotation procedures.", "What happens if a developer accidentally commits a secret?")
];

pub fn question(id: &str, domain: &str, text: &str) -> Value {
    let offset = match id.as_bytes()[0] { b'P' => 0, b'R' => 10, _ => 20 };
    let idx = id[1..].parse::<usize>().unwrap_or(1) - 1 + offset;
    let (explanation, example, follow_up) = HELP[idx];
    json!({"id":id,"domain":domain,"text":text,"explanation":explanation,"answer_example":example,"follow_up":follow_up,
        "evidence_hint":"Use a non-sensitive document name, ticket reference, review date or anonymized description. Do not paste passwords, keys or customer data."})
}

pub fn overview() -> Value {
    json!({"view":"overview","title":"Cybersecurity maturity assessment","domains":["People","Processes","Technology"],
        "modes":[{"id":"quick","label":"Quick baseline","questions":12},{"id":"full","label":"Full assessment","questions":30}],
        "steps":["Choose your scope","Answer one question at a time","Review gaps and a 30/60/90-day plan"],
        "disclaimer":"Independent ENISA-informed self-assessment. Not an official ENISA score or certification.",
        "demo_notice":"Public demo: everyone can access saved assessments. Use fictional data. Render Free storage is temporary."})
}

pub fn get(args: &Value) -> Result<Value, String> {
    let a = super::load(super::arg_str(args,"assessment_id")?)?;
    let qs: Vec<Value> = super::ids(&a.mode).into_iter().map(|(id,domain,text)|question(&id,domain,text)).collect();
    let next = qs.iter().position(|q| !a.answers.contains_key(q["id"].as_str().unwrap()));
    Ok(json!({"view":"questionnaire","assessment_id":a.id,"organization":a.organization,"mode":a.mode,
        "answers":a.answers,"questions":qs,"next_index":next,"answered":a.answers.len(),"total":qs.len(),"demo_notice":overview()["demo_notice"]}))
}

pub fn render(args: &Value) -> Result<Value,String> {
    if args.get("assessment_id").and_then(Value::as_str).is_some() { get(args) } else { Ok(overview()) }
}

pub fn resource() -> Value {
    json!({"uri":URI,"name":"Cyber maturity questionnaire","mimeType":"text/html;profile=mcp-app","description":"Assessment overview and one-question-at-a-time form"})
}

pub fn read(uri: &str) -> Result<Value, String> {
    if uri != URI { return Err("Unknown UI resource".into()); }
    Ok(json!({"contents":[{"uri":URI,"mimeType":"text/html;profile=mcp-app","text":HTML,
        "_meta":{"ui":{"prefersBorder":true,"csp":{"connectDomains":[],"resourceDomains":[]}},
        "openai/widgetDescription":"An assessment form that starts with an overview, then shows one question with answer fields, explanation, examples and follow-up guidance."}}]}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn every_question_has_specific_guidance() {
        for (id,domain,text) in crate::ids("full") {
            let q = question(&id,domain,text);
            assert!(q["explanation"].as_str().unwrap().len()>30);
            assert!(q["answer_example"].as_str().unwrap().len()>30);
            assert!(q["follow_up"].as_str().unwrap().ends_with('?'));
        }
    }
    #[test] fn overview_precedes_creation() {
        assert_eq!(render(&json!({})).unwrap()["view"],"overview");
    }
    #[test] fn registered_resource_is_embedded() {
        assert!(HTML.contains("<html"));
        assert!(read(URI).unwrap()["contents"][0]["text"].as_str().unwrap().contains("Assessment overview"));
        assert!(read("ui://unknown").is_err());
    }
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;
    #[test] fn old_answers_still_deserialize() {
        let answer: crate::Answer = serde_json::from_value(json!({"value":"3","evidence":"Ticket-1"})).unwrap();
        assert!(answer.answer_text.is_none());
        assert!(answer.follow_up_answer.is_none());
    }
    #[test] fn free_text_fields_round_trip() {
        let answer: crate::Answer = serde_json::from_value(json!({"value":"2","answer_text":"Partly implemented","follow_up_answer":"Owner follows up"})).unwrap();
        let saved=serde_json::to_value(answer).unwrap();
        assert_eq!(saved["answer_text"],"Partly implemented");
        assert_eq!(saved["follow_up_answer"],"Owner follows up");
    }
    #[test] fn correct_read_and_write_action_metadata() {
        for t in crate::tools() {
            let write=["create_assessment","record_answer","delete_assessment"].contains(&t["name"].as_str().unwrap());
            assert_eq!(t["annotations"]["readOnlyHint"],!write);
            if t["name"]=="assessment_overview" { assert_eq!(t["_meta"]["ui"]["resourceUri"],URI); }
            else { assert!(t["_meta"]["ui"].get("resourceUri").is_none()); }
        }
    }
}
