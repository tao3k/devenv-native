use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use xiuxian_qianji_client::load_flowhub_scenario_registry;

#[derive(Debug, Clone)]
pub(crate) struct QianjiServerFlowhubState {
    flowhub_root: PathBuf,
}

impl QianjiServerFlowhubState {
    pub(crate) fn new(flowhub_root: PathBuf) -> Self {
        Self { flowhub_root }
    }
}

pub(crate) fn qianji_server_flowhub_router(state: QianjiServerFlowhubState) -> Router {
    Router::new()
        .route("/flowhub/scenarios", get(flowhub_scenarios))
        .with_state(state)
}

pub(crate) fn resolve_qianji_server_flowhub_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(flowhub_root) = explicit {
        return flowhub_root.to_path_buf();
    }
    if let Some(flowhub_root) = env_path("QIANJI_FLOWHUB_ROOT") {
        return flowhub_root;
    }
    if let Some(project_root) = env_path("PRJ_ROOT") {
        return project_root.join("qianji-flowhub");
    }
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("qianji-flowhub")
}

async fn flowhub_scenarios(State(state): State<QianjiServerFlowhubState>) -> Response {
    let flowhub_root = state.flowhub_root;
    let result = tokio::task::spawn_blocking(move || load_flowhub_scenario_registry(&flowhub_root))
        .await
        .map_err(|error| format!("Flowhub registry worker failed: {error}"))
        .and_then(|result| result.map_err(|error| error.to_string()));

    match result {
        Ok(registry) if registry.passed => Json(registry).into_response(),
        Ok(registry) => (StatusCode::SERVICE_UNAVAILABLE, Json(registry)).into_response(),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(QianjiServerFlowhubErrorResponse {
                passed: false,
                error,
            }),
        )
            .into_response(),
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

#[derive(Debug, Serialize)]
struct QianjiServerFlowhubErrorResponse {
    passed: bool,
    error: String,
}
