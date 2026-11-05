mod assessment;
mod content;
mod http;
mod mcp;
mod model;
mod scoring;
mod storage;
#[cfg(test)]
mod tests;
mod ui;

use serde_json::Value;
use std::io::{self, BufRead, Write};
const SUPPORTED_VERSIONS: &[&str] = &["2025-03-26", "2025-06-18", "2025-11-25"];
fn handle(req: &Value) -> Option<Value> {
    mcp::handle(
        req,
        &assessment::Service::new(storage::Repository::new(storage::data_dir())),
    )
}
fn error(id: Value, code: i32, message: impl ToString) -> Value {
    mcp::error(id, code, message)
}
fn stdio_main() {
    let input = io::stdin();
    let mut output = io::stdout().lock();
    for line in input.lock().lines() {
        let line = match line {
            Ok(s) => s,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(v) => handle(&v),
            Err(e) => Some(error(Value::Null, -32700, e)),
        };
        if let Some(v) = reply {
            if writeln!(output, "{v}").is_err() {
                break;
            }
            let _ = output.flush();
        }
    }
}
#[tokio::main]
async fn main() {
    let _ = content::latest(); // Parse and validate the bank once, before accepting requests.
    if std::env::args().any(|a| a == "--stdio") {
        stdio_main()
    } else {
        http::serve().await
    }
}
