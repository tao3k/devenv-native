//! Gateway model-route admission handlers.

use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use xiuxian_llm::model_routing::{
    WendaoChatRouteConfig, WendaoChatRouteInput, WendaoModelDecision, WendaoModelRoutingMode,
    WendaoRouteIntent, wendao_chat_model_route_decision,
    wendao_chat_route_config_with_model_routing_config,
};

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
    /// Active routing mode.
    pub model_routing_mode: &'static str,
    /// Gateway route intent sent to the routing plane.
    pub intent: WendaoRouteIntent,
    /// Gateway-selected model decision.
    pub decision: WendaoModelDecision,
}

/// Admit a chat model route through Gateway-owned routing.
///
/// # Errors
///
/// Returns an admission error when routing configuration is invalid or the
/// configured routing plane cannot produce a model decision.
pub async fn admit_chat_route(
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<ChatModelRouteRequest>,
) -> Result<Json<ChatModelRouteResponse>, StudioApiError> {
    let model_routing = state
        .studio
        .model_routing_config()
        .map_err(|error| StudioApiError::bad_request("MODEL_ROUTE_CONFIG_INVALID", error))?;
    let config =
        wendao_chat_route_config_with_model_routing_config(model_routing.as_ref(), &env_lookup)
            .map_err(|error| StudioApiError::bad_request("MODEL_ROUTE_CONFIG_INVALID", error))?;
    admit_chat_model_route_with_config(config, payload)
        .await
        .map(Json)
}

pub(crate) async fn admit_chat_model_route_with_config(
    config: WendaoChatRouteConfig,
    request: ChatModelRouteRequest,
) -> Result<ChatModelRouteResponse, StudioApiError> {
    let input = WendaoChatRouteInput {
        precision_tier: normalized_or_default(&request.precision_tier, default_precision_tier()),
        privacy_tier: normalized_or_default(&request.privacy_tier, default_privacy_tier()),
        latency_budget_ms: request.latency_budget_ms,
        evidence_profile: normalized_or_default(
            &request.evidence_profile,
            default_evidence_profile(),
        ),
        artifact_refs: request.artifact_refs,
    };
    let decision = wendao_chat_model_route_decision(&config, &input)
        .await
        .map_err(|error| StudioApiError::unavailable("MODEL_ROUTE_ADMISSION_FAILED", error))?;
    let (intent, decision) = decision;
    Ok(ChatModelRouteResponse {
        schema_version: MODEL_ROUTE_CHAT_SCHEMA,
        model_routing_mode: model_routing_mode_label(config.model_routing_mode),
        intent,
        decision,
    })
}

fn env_lookup(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

fn model_routing_mode_label(mode: WendaoModelRoutingMode) -> &'static str {
    match mode {
        WendaoModelRoutingMode::VllmSr => "vllm-sr",
        WendaoModelRoutingMode::Deterministic => "deterministic",
    }
}

fn normalized_or_default(value: &str, default: String) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        default
    } else {
        normalized.to_owned()
    }
}

fn default_precision_tier() -> String {
    WendaoChatRouteInput::default().precision_tier
}

fn default_privacy_tier() -> String {
    WendaoChatRouteInput::default().privacy_tier
}

fn default_latency_budget_ms() -> u64 {
    WendaoChatRouteInput::default().latency_budget_ms
}

fn default_evidence_profile() -> String {
    WendaoChatRouteInput::default().evidence_profile
}
