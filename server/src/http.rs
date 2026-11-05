use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::{env, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone)]
struct App {
    service: Arc<crate::assessment::Service>,
    origins: Arc<Vec<String>>,
    gate: Arc<Mutex<()>>,
}

fn check(app: &App, headers: &HeaderMap) -> Result<(), StatusCode> {
    if let Some(origin) = headers.get("origin") {
        let origin = origin.to_str().map_err(|_| StatusCode::FORBIDDEN)?;
        if !app.origins.iter().any(|x| x == origin) {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    if let Some(version) = headers.get("mcp-protocol-version") {
        if !version
            .to_str()
            .ok()
            .is_some_and(|v| super::SUPPORTED_VERSIONS.contains(&v))
        {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(())
}
fn rejected(status: StatusCode) -> Response {
    status.into_response()
}
async fn no_stream(State(app): State<App>, headers: HeaderMap) -> Response {
    if let Err(s) = check(&app, &headers) {
        return rejected(s);
    }
    let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
    response
        .headers_mut()
        .insert("allow", "POST".parse().unwrap());
    response
}
fn valid(v: &Value) -> bool {
    v.is_object()
        && v.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && (v.get("method").and_then(Value::as_str).is_some()
            || (v.get("id").is_some() && (v.get("result").is_some() || v.get("error").is_some())))
        && v.get("id")
            .is_none_or(|id| id.is_string() || id.is_number())
        && v.get("params")
            .is_none_or(|p| p.is_object() || p.is_array())
}
async fn post(State(app): State<App>, headers: HeaderMap, bytes: Bytes) -> Response {
    if let Err(s) = check(&app, &headers) {
        return rejected(s);
    }
    if !headers
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|s| s.split(';').next().unwrap_or("").trim() == "application/json")
    {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    let accept = headers
        .get("accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if !accept.contains("application/json") || !accept.contains("text/event-stream") {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    }
    let req: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(super::error(Value::Null, -32700, "Parse error")),
            )
                .into_response()
        }
    };
    // Batches were allowed in 2025-03-26, and removed in later revisions.
    let batch = req.is_array();
    let messages = if let Some(v) = req.as_array() {
        if headers
            .get("mcp-protocol-version")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("2025-03-26")
            != "2025-03-26"
            || v.is_empty()
        {
            return StatusCode::BAD_REQUEST.into_response();
        }
        v.clone()
    } else {
        vec![req]
    };
    if messages.iter().any(|v| !valid(v)) {
        return (
            StatusCode::BAD_REQUEST,
            Json(super::error(Value::Null, -32600, "Invalid Request")),
        )
            .into_response();
    }
    // Serialize the entire load/modify/save cycle, including reads and deletes.
    // One process owns the demo files. Keep this gate around all read/modify/write operations.
    let gate = app.gate.clone().lock_owned().await;
    let service = app.service.clone();
    let replies = tokio::task::spawn_blocking(move || {
        let _guard = gate;
        messages
            .iter()
            .filter(|v| v.get("method").is_some())
            .filter_map(|v| crate::mcp::handle(v, &service))
            .collect::<Vec<_>>()
    })
    .await;
    match replies {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(v) if v.is_empty() => StatusCode::ACCEPTED.into_response(),
        Ok(v) if batch => Json(json!(v)).into_response(),
        Ok(mut v) => Json(v.remove(0)).into_response(),
    }
}
fn router(app: App) -> Router {
    Router::new()
        .route(
            "/healthz",
            get(|| async { Json(json!({"status":"ok","version":env!("CARGO_PKG_VERSION")})) }),
        )
        .route("/mcp", get(no_stream).post(post).delete(no_stream))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .with_state(app)
}
pub async fn serve() {
    let origins = env::var("MCP_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    std::fs::create_dir_all(crate::storage::data_dir()).expect("Cannot create data directory");
    let probe = crate::storage::data_dir().join(format!(".startup-{}", uuid::Uuid::new_v4()));
    std::fs::write(&probe, b"probe").expect("Data directory is not writable");
    std::fs::remove_file(probe).expect("Cannot remove storage probe");
    let port = env::var("PORT")
        .unwrap_or_else(|_| "10000".into())
        .parse::<u16>()
        .expect("Invalid PORT");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .expect("Cannot bind HTTP listener");
    eprintln!("SME MCP listening on {host}:{port}; stateless HTTP, PUBLIC DEMO: anonymous shared workspace");
    axum::serve(
        listener,
        router(App {
            service: Arc::new(crate::assessment::Service::new(
                crate::storage::Repository::new(crate::storage::data_dir()),
            )),
            origins: Arc::new(origins),
            gate: Arc::new(Mutex::new(())),
        }),
    )
    .with_graceful_shutdown(async {
        #[cfg(unix)]
        {
            let mut term =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = term.recv() => {} }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }
    })
    .await
    .expect("HTTP server failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    fn app() -> App {
        App {
            service: Arc::new(crate::assessment::Service::new(
                crate::storage::Repository::new(crate::storage::data_dir()),
            )),
            origins: Arc::new(vec!["https://example.com".into()]),
            gate: Arc::new(Mutex::new(())),
        }
    }
    #[tokio::test]
    async fn anonymous_access_and_origin() {
        for (auth, origin, expected) in [
            (false, None, 405),
            (true, Some("https://evil.example"), 403),
            (true, None, 405),
        ] {
            let mut req = Request::builder().uri("/mcp");
            if auth {
                req = req.header("authorization", format!("Bearer {}", "x".repeat(32)));
            }
            if let Some(o) = origin {
                req = req.header("origin", o);
            }
            assert_eq!(
                router(app())
                    .oneshot(req.body(Body::empty()).unwrap())
                    .await
                    .unwrap()
                    .status()
                    .as_u16(),
                expected
            );
        }
    }
    #[tokio::test]
    async fn initialize_and_notification() {
        for (body, expected) in [
            (
                json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
                200,
            ),
            (
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                202,
            ),
            (json!({"method":"ping"}), 400),
        ] {
            let req = Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("authorization", format!("Bearer {}", "x".repeat(32)))
                .header("content-type", "application/json")
                .header("accept", "application/json, text/event-stream")
                .body(Body::from(body.to_string()))
                .unwrap();
            let r = router(app()).oneshot(req).await.unwrap();
            assert_eq!(r.status().as_u16(), expected);
        }
    }
    #[tokio::test]
    async fn simultaneous_stale_writes_only_apply_once() {
        let dir = std::env::temp_dir().join(format!("sme-http-{}", uuid::Uuid::new_v4()));
        let service = Arc::new(crate::assessment::Service::new(
            crate::storage::Repository::new(dir.clone()),
        ));
        let created = service
            .dispatch(
                "create_assessment",
                &json!({"organization":"Concurrent test"}),
            )
            .unwrap();
        let application = router(App {
            service: service.clone(),
            origins: Arc::new(vec![]),
            gate: Arc::new(Mutex::new(())),
        });
        let req = |question| {
            Request::builder().method("POST").uri("/mcp").header("content-type","application/json").header("accept","application/json, text/event-stream")
            .body(Body::from(json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"record_answer","arguments":{"assessment_id":created["assessment_id"],"question_id":question,"value":"3","expected_revision":0}}}).to_string())).unwrap()
        };
        let (a, b) = tokio::join!(
            application.clone().oneshot(req("P01")),
            application.oneshot(req("P02"))
        );
        let mut successes = 0;
        let mut conflicts = 0;
        for response in [a.unwrap(), b.unwrap()] {
            let bytes = axum::body::to_bytes(response.into_body(), 256 * 1024)
                .await
                .unwrap();
            let result: Value = serde_json::from_slice(&bytes).unwrap();
            if result["result"]["isError"] == false {
                successes += 1
            } else if result["result"]["structuredContent"]["error"]["code"] == "revision_conflict"
            {
                conflicts += 1
            }
        }
        assert_eq!((successes, conflicts), (1, 1));
        let stored = service
            .repository
            .load(created["assessment_id"].as_str().unwrap())
            .unwrap();
        assert_eq!(stored.revision, 1);
        assert_eq!(stored.answers.len(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
