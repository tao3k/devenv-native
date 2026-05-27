//! Chat-specific model route helpers.

use super::config::model_routing_config_lookup_value;
use super::{
    VllmSrRouteDecisionClient, WendaoModelDecision, WendaoModelRoutingMode,
    WendaoModelRoutingTomlConfig, WendaoRouteIntent, WendaoRouteSourceKind, WendaoRouteTaskKind,
    wendao_model_routing_mode_with_lookup, wendao_vllm_sr_base_url_with_lookup,
};

/// Provider hint used for chat route admission.
pub const WENDAO_CHAT_ROUTE_PROVIDER_ENV: &str = "WENDAO_CHAT_ROUTE_PROVIDER";
/// Model selected by the Gateway-owned deterministic local route policy.
pub const WENDAO_CHAT_ROUTE_MODEL_ENV: &str = "WENDAO_CHAT_ROUTE_MODEL";
/// Backend profile hint used for chat route admission.
pub const WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV: &str = "WENDAO_CHAT_ROUTE_BACKEND_PROFILE";
/// Default local chat model selected by deterministic Gateway policy.
pub const DEFAULT_WENDAO_CHAT_ROUTE_MODEL: &str = "deepseek-chat";
/// Default Gateway backend profile for OpenAI-compatible chat execution.
pub const DEFAULT_WENDAO_CHAT_ROUTE_BACKEND_PROFILE: &str = "openai-compatible-chat-v1";
const DEFAULT_WENDAO_CHAT_ROUTE_PROVIDER: &str = "deepseek";

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
    /// Model selected by deterministic local Gateway policy.
    pub route_model: String,
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
        route_model: string_value(
            lookup,
            WENDAO_CHAT_ROUTE_MODEL_ENV,
            DEFAULT_WENDAO_CHAT_ROUTE_MODEL,
        ),
        backend_profile: string_value(
            lookup,
            WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV,
            DEFAULT_WENDAO_CHAT_ROUTE_BACKEND_PROFILE,
        ),
        model_routing_mode: wendao_model_routing_mode_with_lookup(lookup)?,
        vllm_sr_base_url: wendao_vllm_sr_base_url_with_lookup(lookup),
    })
}

/// Resolve chat route config from `wendao.toml` first, then env/runtime lookup.
///
/// # Errors
///
/// Returns an error when shared model-routing mode parsing fails.
pub fn wendao_chat_route_config_with_model_routing_config(
    model_routing: Option<&WendaoModelRoutingTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoChatRouteConfig, String> {
    wendao_chat_route_config_with_lookup(&|key| {
        model_routing_config_lookup_value(model_routing, lookup, key)
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
) -> Result<(WendaoRouteIntent, WendaoModelDecision), String> {
    match config.model_routing_mode {
        WendaoModelRoutingMode::Deterministic => {
            let intent = wendao_chat_route_intent(input);
            let decision = deterministic_chat_route_decision(config)?;
            Ok((intent, decision))
        }
        WendaoModelRoutingMode::VllmSr => {
            let provider = chat_route_provider_hint(config)?;
            let intent = wendao_chat_route_intent(input);
            let decision = VllmSrRouteDecisionClient::new(config.vllm_sr_base_url.as_str())
                .decide(&intent, provider.as_str(), config.backend_profile.as_str())
                .await
                .map_err(|error| format!("chat model route admission failed: {error}"))?;
            Ok((intent, decision))
        }
    }
}

fn deterministic_chat_route_decision(
    config: &WendaoChatRouteConfig,
) -> Result<WendaoModelDecision, String> {
    let selected_provider = chat_route_provider_or_default(config);
    let selected_model = config.route_model.trim();
    if selected_model.is_empty() {
        return Err(format!(
            "{WENDAO_CHAT_ROUTE_MODEL_ENV} must be non-empty in deterministic routing mode"
        ));
    }
    let selected_backend_profile = config.backend_profile.trim();
    if selected_backend_profile.is_empty() {
        return Err(format!(
            "{WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV} must be non-empty in deterministic routing mode"
        ));
    }
    Ok(WendaoModelDecision {
        route_id: format!(
            "deterministic:chat:{}:{}",
            sanitize_route_id_part(selected_provider.as_str()),
            sanitize_route_id_part(selected_model),
        ),
        selected_provider,
        selected_model: selected_model.to_owned(),
        selected_backend_profile: selected_backend_profile.to_owned(),
        reasoning_policy: None,
        route_trace: Some("gateway deterministic local route policy".to_owned()),
    })
}

fn chat_route_provider_or_default(config: &WendaoChatRouteConfig) -> String {
    config
        .route_provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || DEFAULT_WENDAO_CHAT_ROUTE_PROVIDER.to_owned(),
            str::to_owned,
        )
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

fn sanitize_route_id_part(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let normalized = out.trim_matches('-');
    if normalized.is_empty() {
        "unknown".to_owned()
    } else {
        normalized.to_owned()
    }
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
