//! HTTP router surface for Qianhuan Zhenfa endpoints.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde_json::{Value, json};
use xiuxian_zhenfa::{MethodRegistry, ZhenfaRouter};

use crate::manifestation::ManifestationManager;

use super::models::{HttpReloadResponse, HttpRenderRequest, HttpRenderResponse};
use super::rpc::{reload_for_rpc, render, render_from_rpc_params};

const QIANHUAN_PREFIX: &str = "/v1/qianhuan";
const QIANHUAN_RENDER_ROUTE: &str = "/v1/qianhuan/render";
const QIANHUAN_RELOAD_ROUTE: &str = "/v1/qianhuan/reload";

/// `xiuxian-qianhuan` adapter mounted into `xiuxian-zhenfa` gateway.
pub struct QianhuanZhenfaRouter {
    manager: Arc<ManifestationManager>,
}

impl QianhuanZhenfaRouter {
    /// Create a router adapter from manifestation manager state.
    #[must_use]
    pub fn new(manager: Arc<ManifestationManager>) -> Self {
        Self { manager }
    }
}

impl ZhenfaRouter for QianhuanZhenfaRouter {
    fn prefix(&self) -> &'static str {
        QIANHUAN_PREFIX
    }

    fn mount(&self, router: Router) -> Router {
        router.merge(
            Router::new()
                .route(QIANHUAN_RENDER_ROUTE, post(render_http))
                .route(QIANHUAN_RELOAD_ROUTE, post(reload_http))
                .route(QIANHUAN_RELOAD_ROUTE, get(reload_http))
                .with_state(Arc::clone(&self.manager)),
        )
    }

    fn register_methods(&self, registry: &mut MethodRegistry) {
        let render_manager = Arc::clone(&self.manager);
        registry.register_fn("qianhuan.render", move |params, _meta| {
            let manager = Arc::clone(&render_manager);
            async move { render_from_rpc_params(&manager, params) }
        });

        let reload_manager = Arc::clone(&self.manager);
        registry.register_fn("qianhuan.reload", move |_params, _meta| {
            let manager = Arc::clone(&reload_manager);
            async move { reload_for_rpc(&manager) }
        });
    }
}

async fn reload_http(
    State(manager): State<Arc<ManifestationManager>>,
) -> Result<Json<HttpReloadResponse>, (StatusCode, Json<Value>)> {
    manager
        .reload_templates_if_changed()
        .map(|reloaded| Json(HttpReloadResponse { reloaded }))
        .map_err(|error| internal_http_error(&error))
}

async fn render_http(
    State(manager): State<Arc<ManifestationManager>>,
    Json(body): Json<HttpRenderRequest>,
) -> Result<Json<HttpRenderResponse>, (StatusCode, Json<Value>)> {
    render(&manager, &body.request)
        .map(|result| Json(HttpRenderResponse { result }))
        .map_err(|error| internal_http_error(&error))
}

fn internal_http_error(error: &anyhow::Error) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "qianhuan operation failed",
            "details": error.to_string()
        })),
    )
}
