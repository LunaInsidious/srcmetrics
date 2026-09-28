//! `srcmetrics serve`: minimal local HTTP API and UI (ADR-0021).

use axum::Router;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use srcmetrics::analyze::analyze_source;
use std::net::SocketAddr;

const UI: &str = include_str!("ui.html");

#[derive(Deserialize)]
struct AnalyzeRequest {
    /// File name; its extension selects the language.
    filename: String,
    source: String,
}

pub fn serve(address: SocketAddr) -> Result<(), String> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| format!("cannot start the async runtime: {e}"))?;
    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .map_err(|e| format!("cannot listen on {address}: {e}; choose another --port"))?;
        let bound = listener.local_addr().map_err(|e| e.to_string())?;
        eprintln!("listening on http://{bound}");
        let app = Router::new()
            .route("/", get(|| async { Html(UI) }))
            .route("/api/analyze", post(analyze));
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("server error: {e}"))
    })
}

async fn analyze(Json(request): Json<AnalyzeRequest>) -> Response {
    // Parsing and metrics are CPU-bound: keep them off the async workers.
    let result =
        tokio::task::spawn_blocking(move || analyze_source(&request.filename, &request.source))
            .await;
    match result {
        Ok(Ok(analysis)) => Json(analysis).into_response(),
        Ok(Err(error)) => error_response(StatusCode::BAD_REQUEST, error.to_string()),
        Err(join_error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("analysis failed: {join_error}"),
        ),
    }
}

fn error_response(status: StatusCode, message: String) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
