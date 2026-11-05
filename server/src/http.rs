use axum::{body::Bytes, extract::{DefaultBodyLimit, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, routing::get, Json, Router};
use serde_json::{json, Value};
use std::{env, sync::Arc};
use subtle::ConstantTimeEq;
use tokio::sync::Mutex;

#[derive(Clone)]
struct App { token: Arc<String>, origins: Arc<Vec<String>>, gate: Arc<Mutex<()>> }

fn check(app: &App, headers: &HeaderMap) -> Result<(), StatusCode> {
    if let Some(origin) = headers.get("origin") {
        let origin = origin.to_str().map_err(|_| StatusCode::FORBIDDEN)?;
        if !app.origins.iter().any(|x| x == origin) { return Err(StatusCode::FORBIDDEN); }
    }
    let supplied = headers.get("authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).unwrap_or("");
    if !bool::from(supplied.as_bytes().ct_eq(app.token.as_bytes())) { return Err(StatusCode::UNAUTHORIZED); }
    if let Some(version) = headers.get("mcp-protocol-version") {
        if !version.to_str().ok().is_some_and(|v| super::SUPPORTED_VERSIONS.contains(&v)) { return Err(StatusCode::BAD_REQUEST); }
    }
    Ok(())
}
fn rejected(status: StatusCode) -> Response {
    let mut response = status.into_response();
    if status == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert("www-authenticate", "Bearer realm=\"sme-maturity\"".parse().unwrap());
    }
    response
}
async fn no_stream(State(app): State<App>, headers: HeaderMap) -> Response {
    if let Err(s) = check(&app, &headers) { return rejected(s); }
    let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
    response.headers_mut().insert("allow", "POST".parse().unwrap());
    response
}
fn valid(v: &Value) -> bool {
    v.is_object() && v.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && (v.get("method").and_then(Value::as_str).is_some()
            || (v.get("id").is_some() && (v.get("result").is_some() || v.get("error").is_some())))
        && v.get("id").is_none_or(|id| id.is_string() || id.is_number())
        && v.get("params").is_none_or(|p| p.is_object() || p.is_array())
}
async fn post(State(app): State<App>, headers: HeaderMap, bytes: Bytes) -> Response {
    if let Err(s) = check(&app, &headers) { return rejected(s); }
    if !headers.get("content-type").and_then(|h| h.to_str().ok()).is_some_and(|s| s.split(';').next().unwrap_or("").trim() == "application/json") {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    let accept = headers.get("accept").and_then(|h| h.to_str().ok()).unwrap_or("");
    if !accept.contains("application/json") || !accept.contains("text/event-stream") {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    }
    let req: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(super::error(Value::Null, -32700, "Parse error".into()))).into_response(),
    };
    // Batches were allowed in 2025-03-26, and removed in later revisions.
    let batch = req.is_array();
    let messages = if let Some(v) = req.as_array() {
        if headers.get("mcp-protocol-version").and_then(|h| h.to_str().ok()).unwrap_or("2025-03-26") != "2025-03-26" || v.is_empty() {
            return StatusCode::BAD_REQUEST.into_response();
        }
        v.clone()
    } else { vec![req] };
    if messages.iter().any(|v| !valid(v)) {
        return (StatusCode::BAD_REQUEST, Json(super::error(Value::Null, -32600, "Invalid Request".into()))).into_response();
    }
    // Serialize the entire load/modify/save cycle, including reads and deletes.
    // This service intentionally uses one process and one persistent disk.
    let gate = app.gate.clone().lock_owned().await;
    let replies = tokio::task::spawn_blocking(move || {
        let _guard = gate;
        messages.iter().filter(|v| v.get("method").is_some()).filter_map(super::handle).collect::<Vec<_>>()
    }).await;
    match replies {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(v) if v.is_empty() => StatusCode::ACCEPTED.into_response(),
        Ok(v) if batch => Json(json!(v)).into_response(),
        Ok(mut v) => Json(v.remove(0)).into_response(),
    }
}
fn router(app: App) -> Router {
    Router::new()
        .route("/healthz", get(|| async { Json(json!({"status":"ok","version":"0.3.0"})) }))
        .route("/mcp", get(no_stream).post(post).delete(no_stream))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .with_state(app)
}
pub async fn serve() {
    let token = env::var("MCP_API_KEY").expect("MCP_API_KEY must be set");
    assert!(token.len() >= 32 && token.is_ascii() && !token.bytes().any(|b| b.is_ascii_whitespace()), "MCP_API_KEY must contain at least 32 non-whitespace ASCII characters");
    let origins = env::var("MCP_ALLOWED_ORIGINS").unwrap_or_default().split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect();
    std::fs::create_dir_all(super::datastore()).expect("Cannot create data directory");
    let probe = super::datastore().join(format!(".startup-{}", uuid::Uuid::new_v4()));
    std::fs::write(&probe, b"probe").expect("Data directory is not writable");
    std::fs::remove_file(probe).expect("Cannot remove storage probe");
    let port = env::var("PORT").unwrap_or_else(|_| "10000".into()).parse::<u16>().expect("Invalid PORT");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await.expect("Cannot bind HTTP listener");
    eprintln!("SME MCP listening on {host}:{port}; stateless HTTP, private shared workspace");
    axum::serve(listener, router(App { token: Arc::new(token), origins: Arc::new(origins), gate: Arc::new(Mutex::new(())) }))
        .with_graceful_shutdown(async {
            #[cfg(unix)]
            { let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap(); tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = term.recv() => {} } }
            #[cfg(not(unix))]
            { let _ = tokio::signal::ctrl_c().await; }
        }).await.expect("HTTP server failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    fn app() -> App { App { token: Arc::new("x".repeat(32)), origins: Arc::new(vec!["https://example.com".into()]), gate: Arc::new(Mutex::new(())) } }
    #[tokio::test]
    async fn auth_and_origin() {
        for (auth, origin, expected) in [(false, None, 401), (true, Some("https://evil.example"), 403), (true, None, 405)] {
            let mut req = Request::builder().uri("/mcp");
            if auth { req = req.header("authorization", format!("Bearer {}", "x".repeat(32))); }
            if let Some(o) = origin { req = req.header("origin", o); }
            assert_eq!(router(app()).oneshot(req.body(Body::empty()).unwrap()).await.unwrap().status().as_u16(), expected);
        }
    }
    #[tokio::test]
    async fn initialize_and_notification() {
        for (body, expected) in [(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}), 200), (json!({"jsonrpc":"2.0","method":"notifications/initialized"}), 202), (json!({"method":"ping"}), 400)] {
            let req = Request::builder().method("POST").uri("/mcp")
                .header("authorization", format!("Bearer {}", "x".repeat(32)))
                .header("content-type", "application/json").header("accept", "application/json, text/event-stream")
                .body(Body::from(body.to_string())).unwrap();
            let r = router(app()).oneshot(req).await.unwrap();
            assert_eq!(r.status().as_u16(), expected);
        }
    }
}
