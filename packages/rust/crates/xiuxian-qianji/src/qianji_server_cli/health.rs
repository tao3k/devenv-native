use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

#[derive(Debug, Clone)]
pub(crate) struct QianjiServerHealthState {
    valkey_url: String,
}

impl QianjiServerHealthState {
    pub(crate) fn new(valkey_url: String) -> Self {
        Self { valkey_url }
    }
}

pub(crate) fn qianji_server_health_router(state: QianjiServerHealthState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/capabilities", get(capabilities))
        .with_state(state)
}

async fn healthz() -> Json<QianjiServerHealthResponse> {
    Json(QianjiServerHealthResponse {
        status: "ok",
        service: "qianji-server",
        checkpoint_default_backend: "valkey",
        valkey_configured: true,
    })
}

async fn capabilities() -> Json<QianjiServerCapabilitiesResponse> {
    Json(QianjiServerCapabilitiesResponse {
        service: "qianji-server",
        checkpoint_default_backend: "valkey",
        capabilities: qianji_server_capabilities(),
    })
}

fn qianji_server_capabilities() -> Vec<&'static str> {
    let mut capabilities = vec![
        "health.healthz",
        "health.readyz",
        "flowhub.scenarios",
        "bpmn.workflow.start",
        "bpmn.workflow.resume",
        "bpmn.workflow.status",
        "bpmn.workflow.cancel",
        "bpmn.workflow.events.poll",
        "bpmn.workflow.task.complete",
        "bpmn.workflow.task.complete-batch",
        "bpmn.workflow.task.fail",
        "bpmn.workflow.activity-evidence",
        "bpmn.workflow.task.claim",
        "bpmn.workflow.task.release",
        "qianji.control.workflow-source.admit",
        "qianji.control.bpmn-source.admit",
        "qianji.control.bpmn-source",
        "qianji.control.execution-graph",
        "qianji.control.history",
        "qianji.control.summary",
        "qianji.control.recovery",
        "qianji.control.diagnostics",
    ];
    #[cfg(feature = "duckdb")]
    capabilities.push("qianji.control.run-console.arrow-flight");
    capabilities.extend(qianji_server_valkey_capabilities());
    capabilities
}

#[cfg(all(feature = "duckdb", feature = "valkey", feature = "qianji-full"))]
fn qianji_server_valkey_capabilities() -> [&'static str; 3] {
    [
        "qianji.control.recovery.apply",
        "qianji.control.worker.openai-compatible-llm.run",
        "qianji.control.worker.openai-compatible-llm.run-and-complete",
    ]
}

#[cfg(all(
    feature = "valkey",
    not(all(feature = "duckdb", feature = "qianji-full"))
))]
fn qianji_server_valkey_capabilities() -> [&'static str; 1] {
    ["qianji.control.recovery.apply"]
}

#[cfg(not(feature = "valkey"))]
fn qianji_server_valkey_capabilities() -> [&'static str; 0] {
    []
}

async fn readyz(State(state): State<QianjiServerHealthState>) -> Response {
    match check_valkey_ready(&state.valkey_url).await {
        Ok(()) => (
            StatusCode::OK,
            Json(QianjiServerReadinessResponse {
                status: "ready",
                service: "qianji-server",
                checkpoint_default_backend: "valkey",
                valkey: QianjiServerValkeyReadiness {
                    status: "ready",
                    message: None,
                },
            }),
        )
            .into_response(),
        Err(message) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(QianjiServerReadinessResponse {
                status: "not_ready",
                service: "qianji-server",
                checkpoint_default_backend: "valkey",
                valkey: QianjiServerValkeyReadiness {
                    status: "not_ready",
                    message: Some(message),
                },
            }),
        )
            .into_response(),
    }
}

pub(crate) async fn check_valkey_ready(valkey_url: &str) -> Result<(), String> {
    let client = redis::Client::open(valkey_url)
        .map_err(|error| format!("failed to open Valkey client: {error}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|error| format!("failed to connect to Valkey: {error}"))?;
    let response: String = redis::cmd("PING")
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("failed to ping Valkey: {error}"))?;
    if response == "PONG" {
        Ok(())
    } else {
        Err(format!("unexpected Valkey ping response `{response}`"))
    }
}

#[derive(Debug, Serialize)]
struct QianjiServerHealthResponse {
    status: &'static str,
    service: &'static str,
    checkpoint_default_backend: &'static str,
    valkey_configured: bool,
}

#[derive(Debug, Serialize)]
struct QianjiServerCapabilitiesResponse {
    service: &'static str,
    checkpoint_default_backend: &'static str,
    capabilities: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct QianjiServerReadinessResponse {
    status: &'static str,
    service: &'static str,
    checkpoint_default_backend: &'static str,
    valkey: QianjiServerValkeyReadiness,
}

#[derive(Debug, Serialize)]
struct QianjiServerValkeyReadiness {
    status: &'static str,
    message: Option<String>,
}
