//! Chat-specific model route helpers.

use super::{
    VllmSrRouteDecisionClient, WendaoModelDecision, WendaoModelRoutingMode, WendaoRouteIntent,
    WendaoRouteSourceKind, WendaoRouteTaskKind, wendao_model_routing_mode_with_lookup,
    wendao_vllm_sr_base_url_with_lookup,
};

/// Provider hint used for chat route admission.
pub const WENDAO_CHAT_ROUTE_PROVIDER_ENV: &str = "WENDAO_CHAT_ROUTE_PROVIDER";
/// Backend profile hint used for chat route admission.
pub const WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV: &str = "WENDAO_CHAT_ROUTE_BACKEND_PROFILE";
/// Default Gateway backend profile for OpenAI-compatible chat execution.
pub const DEFAULT_WENDAO_CHAT_ROUTE_BACKEND_PROFILE: &str = "openai-compatible-chat-v1";

const CHAT_ROUTE_TASK_KIND: &str = "chat";
const CHAT_ROUTE_MODALITY: &str = "text";
const CHAT_ROUTE_SOURCE_KIND: &str = "conversation";
const CHAT_ROUTE_PRECISION_TIER: &str = "high";
const CHAT_ROUTE_PRIVACY_TIER: &str = "private";
const CHAT_ROUTE_EVIDENCE_PROFILE: &str = "local-knowledge-chat";
const CHAT_ROUTE_LATENCY_BUDGET_MS: u64 = 60_000;

/// Chat route controls resolved from Gateway configuration or request context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoChatRouteConfig {
    /// Provider hint required by the current vLLM-SR probe integration.
    pub route_provider: Option<String>,
    /// Gateway backend profile selected for chat execution.
    pub backend_profile: String,
    /// Active routing mode.
    pub model_routing_mode: WendaoModelRoutingMode,
    /// vLLM-SR base URL.
    pub vllm_sr_base_url: String,
}

/// Chat route request facts supplied by Gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoChatRouteInput {
    /// Precision tier required by Gateway.
    pub precision_tier: String,
    /// Privacy tier used by routing policy.
    pub privacy_tier: String,
    /// Latency budget in milliseconds.
    pub latency_budget_ms: u64,
    /// Evidence profile for this chat request.
    pub evidence_profile: String,
    /// Artifact or evidence references available to the model.
    pub artifact_refs: Vec<String>,
}

impl Default for WendaoChatRouteInput {
    fn default() -> Self {
        Self {
            precision_tier: CHAT_ROUTE_PRECISION_TIER.to_owned(),
            privacy_tier: CHAT_ROUTE_PRIVACY_TIER.to_owned(),
            latency_budget_ms: CHAT_ROUTE_LATENCY_BUDGET_MS,
            evidence_profile: CHAT_ROUTE_EVIDENCE_PROFILE.to_owned(),
            artifact_refs: Vec::new(),
        }
    }
}

/// Resolve chat route config using an injectable lookup.
///
/// # Errors
///
/// Returns an error when shared model-routing mode parsing fails.
pub fn wendao_chat_route_config_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoChatRouteConfig, String> {
    Ok(WendaoChatRouteConfig {
        route_provider: optional_string_value(lookup, WENDAO_CHAT_ROUTE_PROVIDER_ENV),
        backend_profile: string_value(
            lookup,
            WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV,
            DEFAULT_WENDAO_CHAT_ROUTE_BACKEND_PROFILE,
        ),
        model_routing_mode: wendao_model_routing_mode_with_lookup(lookup)?,
        vllm_sr_base_url: wendao_vllm_sr_base_url_with_lookup(lookup),
    })
}

/// Build a Gateway-owned chat route intent.
#[must_use]
pub fn wendao_chat_route_intent(input: &WendaoChatRouteInput) -> WendaoRouteIntent {
    WendaoRouteIntent {
        task_kind: WendaoRouteTaskKind::new(CHAT_ROUTE_TASK_KIND),
        modality: CHAT_ROUTE_MODALITY.to_owned(),
        source_kind: WendaoRouteSourceKind::new(CHAT_ROUTE_SOURCE_KIND),
        precision_tier: input.precision_tier.clone(),
        privacy_tier: input.privacy_tier.clone(),
        latency_budget_ms: input.latency_budget_ms,
        evidence_profile: input.evidence_profile.clone(),
        artifact_refs: input.artifact_refs.clone(),
    }
}

/// Admit one chat request through the configured model routing mode.
///
/// # Errors
///
/// Returns an infrastructure admission error when vLLM-SR mode is active but
/// no provider hint is configured, or when the route probe fails.
pub async fn wendao_chat_model_route_decision(
    config: &WendaoChatRouteConfig,
    input: &WendaoChatRouteInput,
) -> Result<Option<(WendaoRouteIntent, WendaoModelDecision)>, String> {
    match config.model_routing_mode {
        WendaoModelRoutingMode::Deterministic => Ok(None),
        WendaoModelRoutingMode::VllmSr => {
            let provider = chat_route_provider_hint(config)?;
            let intent = wendao_chat_route_intent(input);
            let decision = VllmSrRouteDecisionClient::new(config.vllm_sr_base_url.as_str())
                .decide(&intent, provider.as_str(), config.backend_profile.as_str())
                .await
                .map_err(|error| format!("chat model route admission failed: {error}"))?;
            Ok(Some((intent, decision)))
        }
    }
}

fn chat_route_provider_hint(config: &WendaoChatRouteConfig) -> Result<String, String> {
    config
        .route_provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            format!("WENDAO_MODEL_ROUTING_MODE=vllm-sr requires {WENDAO_CHAT_ROUTE_PROVIDER_ENV}")
        })
}

fn string_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: &'static str,
) -> String {
    lookup(key)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

fn optional_string_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
) -> Option<String> {
    lookup(key)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
