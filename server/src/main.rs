use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use uuid::Uuid;

const PEOPLE: [&str;10] = [
    "Is someone explicitly accountable for cybersecurity decisions and spend?",
    "Are all staff and contractors taught practical phishing/account-security basics at onboarding?",
    "Can staff report suspicious messages or incidents quickly without guessing whom to contact?",
    "Do owners of privileged/financial accounts know how to recognize and challenge suspicious requests?",
    "Are contractor access rights reviewed when their work changes or ends?",
    "Are security responsibilities documented for outsourced IT or SaaS providers?",
    "Is security discussed with company leadership when changes or purchases are proposed?",
    "Do teams practice an incident scenario (e.g., account compromise) at least annually?",
    "Do staff understand rules for handling customer, financial and proprietary data?",
    "Is security training adjusted after incidents, process changes, or identified weaknesses?",
];
const PROCESSES: [&str;10] = [
    "Is there an up-to-date list of critical business services, systems and data owners?",
    "Is there a recurring process for ranking cyber risks by business impact and likelihood?",
    "Are key legal, privacy, contractual and customer security obligations tracked?",
    "Is there a documented incident response contact list and initial containment process?",
    "Are access requests/approvals and offboarding consistently documented?",
    "Is there a backup and recovery plan with defined recovery needs for critical services?",
    "Are material cloud/SaaS vendors evaluated and reassessed for security dependencies?",
    "Is there a routine to patch, remediate, or accept risks with named owners and deadlines?",
    "Are security-impacting changes reviewed, tested and logged before release?",
    "Are recovery and incident response plans tested, and lessons incorporated into improvements?",
];
const TECHNOLOGY: [&str;10] = [
    "Is phishing-resistant MFA enabled on privileged accounts and MFA on other critical accounts wherever supported?",
    "Are endpoints and operating systems supported, protected and updated?",
    "Are essential data and settings backed up, with restore tests completed?",
    "Are admin privileges minimized and accounts uniquely attributable to individuals?",
    "Is there a maintained inventory of cloud, SaaS, devices and critical external services?",
    "Are sensitive data encrypted in transit and at rest where applicable?",
    "Are critical cloud/SaaS configurations reviewed against secure defaults?",
    "Do critical systems produce accessible security logs and actionable alerts?",
    "Are publicly exposed systems/dependencies scanned and findings prioritized and remediated?",
    "Are secrets kept in managed stores rather than source code, personal chats or shared documents?",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Answer { value: String, #[serde(default)] evidence: Option<String>, #[serde(default)] owner: Option<String>, #[serde(default)] na_justification: Option<String>, #[serde(default)] verified_at: Option<String> }
#[derive(Debug, Serialize, Deserialize)]
struct Assessment { id:String, organization:String, mode:String, #[serde(default)] answers:BTreeMap<String,Answer> }

fn ids(mode:&str)->Vec<(String,&'static str,&'static str)> {
    let max=if mode=="quick" {4} else {10};
    let mut out=Vec::new();
    for (prefix,domain,questions) in [("P","People",&PEOPLE),("R","Processes",&PROCESSES),("T","Technology",&TECHNOLOGY)] {
        for (idx,q) in questions.iter().take(max).enumerate() {out.push((format!("{}{:02}",prefix,idx+1),domain,*q));}
    }
    out
}
fn datastore()->PathBuf { env::var_os("ENISA_MCP_DATA_DIR").map(PathBuf::from).unwrap_or_else(||PathBuf::from("./data")) }
fn file_for(id:&str)->Result<PathBuf,String>{
    Uuid::parse_str(id).map_err(|_|"Invalid assessment_id UUID".to_string())?;
    Ok(datastore().join(format!("{}.json",id)))
}
fn load(id:&str)->Result<Assessment,String>{let f=file_for(id)?;let bytes=fs::read(f).map_err(|_|"Assessment not found".to_string())?;serde_json::from_slice(&bytes).map_err(|e|format!("Invalid stored record: {e}"))}
fn save(a:&Assessment)->Result<(),String>{
    let dir=datastore();fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let dest=file_for(&a.id)?;let tmp=dir.join(format!(".{}.{}.tmp",a.id,Uuid::new_v4()));
    let bytes=serde_json::to_vec_pretty(a).map_err(|e|e.to_string())?;
    {let mut f=OpenOptions::new().write(true).create_new(true).open(&tmp).map_err(|e|e.to_string())?;f.write_all(&bytes).map_err(|e|e.to_string())?;f.sync_all().map_err(|e|e.to_string())?;}
    fs::rename(&tmp,dest).map_err(|e|e.to_string())
}
fn arg_str<'a>(args:&'a Value,key:&str)->Result<&'a str,String>{args.get(key).and_then(Value::as_str).ok_or_else(||format!("Missing string field: {key}"))}
fn optional_string(args:&Value,key:&str)->Option<String>{args.get(key).and_then(Value::as_str).map(|s|s.trim().to_string()).filter(|s|!s.is_empty())}
fn question_bank(args:&Value)->Result<Value,String>{
    let mode=args.get("mode").and_then(Value::as_str).unwrap_or("full");if mode!="full"&&mode!="quick" {return Err("mode must be quick or full".into());}
    Ok(json!({"mode":mode,"official_enisa":false,"score_scale":{"0":"Absent","1":"Ad hoc","2":"Partial","3":"Consistent","4":"Measured, tested, improved","U":"Unknown","NA":"Not applicable (requires justification)"},"questions":ids(mode).into_iter().map(|(id,domain,text)|json!({"id":id,"domain":domain,"text":text})).collect::<Vec<_>>()}))
}
fn create(args:&Value)->Result<Value,String>{
    let organization=arg_str(args,"organization")?.trim();if organization.is_empty()||organization.len()>160{return Err("organization must be 1–160 chars".into())}
    let mode=args.get("mode").and_then(Value::as_str).unwrap_or("quick");if mode!="quick"&&mode!="full"{return Err("mode must be quick or full".into())}
    let assessment=Assessment{id:Uuid::new_v4().to_string(),organization:organization.to_string(),mode:mode.into(),answers:BTreeMap::new()};save(&assessment)?;
    Ok(json!({"assessment_id":assessment.id,"organization":organization,"mode":mode,"questions_total":ids(mode).len()}))
}
fn record(args:&Value)->Result<Value,String>{
    let mut a=load(arg_str(args,"assessment_id")?)?;let id=arg_str(args,"question_id")?.to_uppercase();
    if !ids(&a.mode).iter().any(|(q,_,_)|q==&id){return Err(format!("Question {id} is not in this assessment mode"))}
    let val=arg_str(args,"value")?.trim().to_uppercase();
    if !["0","1","2","3","4","U","NA"].contains(&val.as_str()){return Err("value must be 0–4, U, or NA".into())}
    let justification=optional_string(args,"na_justification");if val=="NA"&&justification.is_none(){return Err("NA requires na_justification".into())}
    let answer=Answer{value:val,evidence:optional_string(args,"evidence"),owner:optional_string(args,"owner"),na_justification:justification,verified_at:optional_string(args,"verified_at")};
    a.answers.insert(id.clone(),answer);save(&a)?;Ok(json!({"assessment_id":a.id,"question_id":id,"saved":true,"answered":a.answers.len(),"total":ids(&a.mode).len()}))
}
fn maturity(score:f64)->&'static str {if score<25.0{"Initial"}else if score<50.0{"Developing"}else if score<75.0{"Managed"}else{"Improving"}}
fn risk_priority(id:&str)->(&'static str,&'static str){
    match id {"T01"|"T03"|"T10"|"R04"|"R06"|"R05"=> ("High","30 days"),"T02"|"T04"|"T07"|"T09"|"R01"|"R08"|"P01"|"P05"=> ("High","30 days"),"P02"|"P03"|"P04"|"R02"|"R03"|"T05"|"T06"|"T08"=> ("Medium","60 days"),_=> ("Medium","90 days")}
}
fn calculate(args:&Value)->Result<Value,String>{
    let a=load(arg_str(args,"assessment_id")?)?;
    let qs=ids(&a.mode);let mut domains=Vec::new();let mut scores=Vec::new();let mut gaps=Vec::new();
    for domain in ["People","Processes","Technology"] {
        let subset:Vec<_>=qs.iter().filter(|(_,d,_)|*d==domain).collect();
        let mut sum=0u32;let mut n=0u32;let mut unknown=0u32;let mut na=0u32;
        for (id,_,question) in subset {
            match a.answers.get(id) {
                None => unknown+=1,
                Some(x) if x.value=="U"=>unknown+=1,
                Some(x) if x.value=="NA"=>na+=1,
                Some(x)=>{
                    let score:u32=x.value.parse().map_err(|_|format!("Bad stored score on {id}"))?;
                    n+=1;sum+=score;
                    if score<3 {
                        let (priority,target)=risk_priority(id);
                        gaps.push(json!({"question_id":id,"domain":domain,"question":question,"observed_score":score,"priority":priority,"target":target,"action":format!("Establish and verify control: {}",question),"evidence_status":if x.evidence.is_some(){"self-reported with evidence reference"}else{"self-reported without evidence"},"owner":x.owner,"verification_needed":true}));
                    }
                }
            }
        }
        let coverage=if n+unknown>0{(n as f64)*100.0/((n+unknown) as f64)}else{100.0};
        let score=if n>0{Some((sum as f64)*100.0/(4.0*n as f64))}else{None};
        let provisional=coverage<60.0||n<3;
        if !provisional{scores.push(score.unwrap());}
        domains.push(json!({"domain":domain,"score_pct":score.map(round1),"coverage_pct":round1(coverage),"numeric":n,"unknown_or_unanswered":unknown,"justified_na":na,"provisional":provisional}));
    }
    let overall=if scores.len()==3{Some(scores.iter().sum::<f64>()/3.0)}else{None};
    gaps.sort_by_key(|v|{(if v["priority"]=="High" {0}else{1},v["observed_score"].as_u64().unwrap_or(9))});
    Ok(json!({"assessment_id":a.id,"organization":a.organization,"mode":a.mode,"official_enisa":false,"assessment_type":"unverified self assessment","domains":domains,"overall_index_pct":overall.map(round1),"plugin_defined_maturity":overall.map(maturity),"gaps":gaps,"needs_verification":qs.iter().filter(|(id,_,_)|a.answers.get(id).map_or(true,|x|x.value=="U")).map(|(id,_,text)|json!({"question_id":id,"question":text,"status":"unknown"})).collect::<Vec<_>>(),"caveat":"Plugin-defined scoring, not official ENISA grading. A documented evidence reference does not itself verify a control. Quick/full scores are not directly comparable. Priorities are templates requiring business-context validation."}))
}
fn round1(x:f64)->f64{(x*10.0).round()/10.0}
fn list_assessments()->Result<Value,String>{
    let root=datastore();if !root.exists(){return Ok(json!({"assessments":[]}))}
    let mut out=Vec::new();for entry in fs::read_dir(root).map_err(|e|e.to_string())? {
        let path=entry.map_err(|e|e.to_string())?.path();if path.extension().and_then(|s|s.to_str())!=Some("json"){continue}
        if let Ok(bytes)=fs::read(&path){if let Ok(a)=serde_json::from_slice::<Assessment>(&bytes){out.push(json!({"assessment_id":a.id,"organization":a.organization,"mode":a.mode,"answered":a.answers.len()}));}}
    }
    Ok(json!({"assessments":out}))
}
fn delete(args:&Value)->Result<Value,String>{let path=file_for(arg_str(args,"assessment_id")?)?;fs::remove_file(path).map_err(|_|"Assessment not found or cannot delete".to_string())?;Ok(json!({"deleted":true}))}
fn schema_tool(name:&str,desc:&str,schema:Value)->Value{json!({"name":name,"description":desc,"inputSchema":schema,"securitySchemes":[{"type":"noauth"}]})}
fn tools()->Vec<Value>{vec![
    schema_tool("get_question_bank","Get the original, unofficial People/Processes/Technology questionnaire (12 or 30 items).",json!({"type":"object","properties":{"mode":{"type":"string","enum":["quick","full"]}},"additionalProperties":false})),
    schema_tool("create_assessment","Create a local persistent cyber maturity assessment.",json!({"type":"object","properties":{"organization":{"type":"string"},"mode":{"type":"string","enum":["quick","full"]}},"required":["organization"],"additionalProperties":false})),
    schema_tool("record_answer","Record an answer, reference to evidence, owner and NA justification.",json!({"type":"object","properties":{"assessment_id":{"type":"string"},"question_id":{"type":"string"},"value":{"type":"string","enum":["0","1","2","3","4","U","NA"]},"evidence":{"type":"string"},"owner":{"type":"string"},"na_justification":{"type":"string"},"verified_at":{"type":"string"}},"required":["assessment_id","question_id","value"],"additionalProperties":false})),
    schema_tool("calculate_maturity","Compute domain scores, coverage, caveats and a prioritized 30/60/90-day gap plan.",json!({"type":"object","properties":{"assessment_id":{"type":"string"}},"required":["assessment_id"],"additionalProperties":false})),
    schema_tool("list_assessments","List assessments in the server's local storage.",json!({"type":"object","properties":{},"additionalProperties":false})),
    schema_tool("delete_assessment","Delete one assessment from local storage.",json!({"type":"object","properties":{"assessment_id":{"type":"string"}},"required":["assessment_id"],"additionalProperties":false}))
]}
fn response(id:Value,result:Value)->Value{json!({"jsonrpc":"2.0","id":id,"result":result})}
fn error(id:Value,code:i32,message:String)->Value{json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})}
fn handle(req:&Value)->Option<Value>{
    let id=req.get("id").cloned().unwrap_or(Value::Null);let method=req.get("method").and_then(Value::as_str).unwrap_or("");
    if req.get("id").is_none(){return None;}
    match method {
        "initialize"=>Some(response(id,json!({"protocolVersion":req.pointer("/params/protocolVersion").and_then(Value::as_str).filter(|v| SUPPORTED_VERSIONS.contains(v)).unwrap_or("2025-11-25"),"capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"enisa-sme-mcp","version":"0.3.1"}}))),
        "ping"=>Some(response(id,json!({}))),
        "tools/list"=>Some(response(id,json!({"tools":tools()}))),
        "tools/call"=>{
            let p=req.get("params").unwrap_or(&Value::Null);let name=p.get("name").and_then(Value::as_str).unwrap_or("");let a=p.get("arguments").unwrap_or(&Value::Null);
            let out=match name {"get_question_bank"=>question_bank(a),"create_assessment"=>create(a),"record_answer"=>record(a),"calculate_maturity"=>calculate(a),"list_assessments"=>list_assessments(),"delete_assessment"=>delete(a),_=>Err(format!("Unknown tool: {name}"))};
            Some(response(id,match out{Ok(v)=>json!({"content":[{"type":"text","text":v.to_string()}],"structuredContent":v,"isError":false}),Err(e)=>json!({"content":[{"type":"text","text":e}],"isError":true})}))
        },
        _=>Some(error(id,-32601,format!("Method not found: {method}")))
    }
}
fn stdio_main(){
    let stdin=io::stdin();let mut stdout=io::stdout().lock();
    for line in stdin.lock().lines(){match line{
        Ok(line)=>{if line.trim().is_empty(){continue}let reply=match serde_json::from_str::<Value>(&line){Ok(v)=>handle(&v),Err(e)=>Some(error(Value::Null,-32700,e.to_string()))};
        if let Some(v)=reply{if writeln!(stdout,"{}",v).is_err(){break}let _=stdout.flush();}},
        Err(e)=>{eprintln!("MCP input error: {e}");break}
    }}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn question_counts(){assert_eq!(ids("quick").len(),12);assert_eq!(ids("full").len(),30);}
    #[test]fn maturity_thresholds(){assert_eq!(maturity(24.9),"Initial");assert_eq!(maturity(50.),"Managed");}
    #[test]fn initialize_works(){let req=json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}});assert_eq!(handle(&req).unwrap()["result"]["serverInfo"]["name"],"enisa-sme-mcp");}
}

mod http;
const SUPPORTED_VERSIONS: &[&str] = &["2025-03-26", "2025-06-18", "2025-11-25"];
#[tokio::main]
async fn main() {
    if env::args().any(|a| a == "--stdio") { stdio_main(); }
    else { http::serve().await; }
}
