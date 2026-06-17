//! Gateway model-route admission handlers.

use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::studio::router::{GatewayState, StudioApiError};

/// Gateway route used by UI consumers to admit chat model routing.
pub const MODEL_ROUTE_CHAT_ROUTE: &str = "/api/model-route/chat";
const MODEL_ROUTE_CHAT_SCHEMA: &str = "xiuxian_wendao.model_route_chat_admission.v1";

/// Chat route admission request supplied by UI or API consumers.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatModelRouteRequest {
    /// Precision tier requested by the chat surface.
    #[serde(default = "default_precision_tier")]
    pub precision_tier: String,
    /// Privacy tier requested by the chat surface.
    #[serde(default = "default_privacy_tier")]
    pub privacy_tier: String,
    /// Latency budget in milliseconds.
    #[serde(default = "default_latency_budget_ms")]
    pub latency_budget_ms: u64,
    /// Evidence profile for local knowledge chat.
    #[serde(default = "default_evidence_profile")]
    pub evidence_profile: String,
    /// Artifact references already admitted by Gateway.
    #[serde(default)]
    pub artifact_refs: Vec<String>,
}

/// Chat route admission response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatModelRouteResponse {
    /// Stable response schema identifier.
    pub schema_version: &'static str,
    /// Retired route status.
    pub status: &'static str,
    /// Operator-facing migration message.
    pub message: &'static str,
}

/// Admit a chat model route through Gateway-owned routing.
///
/// # Errors
///
/// Returns an admission error when routing configuration is invalid or the
/// configured routing plane cannot produce a model decision.
pub async fn admit_chat_route(
    State(_state): State<Arc<GatewayState>>,
    Json(_payload): Json<ChatModelRouteRequest>,
) -> Result<Json<ChatModelRouteResponse>, StudioApiError> {
    Err(StudioApiError::unavailable(
        "MODEL_ROUTE_RETIRED",
        "model routing moved to marlin-agent-core language/provider services",
    ))
}

pub(crate) async fn admit_chat_model_route_with_config(
    _config: (),
    _request: ChatModelRouteRequest,
) -> Result<ChatModelRouteResponse, StudioApiError> {
    Ok(ChatModelRouteResponse {
        schema_version: MODEL_ROUTE_CHAT_SCHEMA,
        status: "retired",
        message: "model routing moved to marlin-agent-core language/provider services",
    })
}

fn default_precision_tier() -> String {
    "standard".to_owned()
}

fn default_privacy_tier() -> String {
    "private".to_owned()
}

fn default_latency_budget_ms() -> u64 {
    30_000
}

fn default_evidence_profile() -> String {
    "local-knowledge".to_owned()
}
