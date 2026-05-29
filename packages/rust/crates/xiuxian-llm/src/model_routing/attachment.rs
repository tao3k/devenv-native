//! Attachment-specific model route helpers.

use super::config::model_routing_config_lookup_value;
use super::{
    VllmSrRouteDecisionClient, WendaoModelDecision, WendaoModelRoutingMode,
    WendaoModelRoutingTomlConfig, WendaoRouteIntent, WendaoRouteSourceKind, WendaoRouteTaskKind,
    wendao_model_routing_mode_with_lookup, wendao_vllm_sr_base_url_with_lookup,
};

/// Provider hint for audio transcript attachment route admission.
pub const WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER_ENV: &str =
    "WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER";
/// Model selected by deterministic audio transcript routing.
pub const WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL_ENV: &str = "WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL";
/// Backend profile selected by audio transcript routing.
pub const WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE_ENV: &str =
    "WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE";
/// Provider hint for image extraction attachment route admission.
pub const WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER_ENV: &str = "WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER";
/// Model selected by deterministic image extraction routing.
pub const WENDAO_IMAGE_EXTRACT_ROUTE_MODEL_ENV: &str = "WENDAO_IMAGE_EXTRACT_ROUTE_MODEL";
/// Backend profile selected by image extraction routing.
pub const WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE_ENV: &str =
    "WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE";

/// Default provider used by deterministic attachment routing.
pub const DEFAULT_WENDAO_ATTACHMENT_ROUTE_PROVIDER: &str = "openrouter";
/// Default hosted audio transcript backend profile.
pub const DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE: &str =
    "hosted-audio-transcript-v1";
/// Default hosted audio transcript model.
pub const DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL: &str = "qwen/qwen3-asr-flash-2026-02-10";
/// Default hosted image extraction backend profile.
pub const DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE: &str = "hosted-vlm-image-extract-v1";
/// Default hosted image extraction model.
pub const DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_MODEL: &str = "qwen/qwen3-vl-8b-instruct";

/// Generic attachment route controls resolved by Gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoAttachmentRouteConfig {
    /// Provider hint for route admission.
    pub route_provider: Option<String>,
    /// Model selected by deterministic Gateway policy.
    pub route_model: String,
    /// Backend profile selected for execution.
    pub backend_profile: String,
    /// Active routing mode.
    pub model_routing_mode: WendaoModelRoutingMode,
    /// vLLM-SR base URL.
    pub vllm_sr_base_url: String,
}

/// Attachment route request facts supplied by Gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoAttachmentRouteInput {
    /// Gateway task kind.
    pub task_kind: WendaoRouteTaskKind,
    /// Source modality.
    pub modality: String,
    /// Source kind.
    pub source_kind: WendaoRouteSourceKind,
    /// Precision tier required by Gateway.
    pub precision_tier: String,
    /// Privacy tier used by route policy.
    pub privacy_tier: String,
    /// Latency budget in milliseconds.
    pub latency_budget_ms: u64,
    /// Evidence profile for this attachment execution.
    pub evidence_profile: String,
    /// Artifact references available to the selected backend.
    pub artifact_refs: Vec<String>,
}

/// Resolve audio transcript route config using an injectable lookup.
///
/// # Errors
///
/// Returns an error when shared model-routing mode parsing fails.
pub fn wendao_audio_transcript_route_config_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoAttachmentRouteConfig, String> {
    wendao_attachment_route_config_with_lookup(
        lookup,
        WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER_ENV,
        WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL_ENV,
        DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL,
        WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE_ENV,
        DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE,
    )
}

/// Resolve audio transcript route config from `wendao.toml` first, then env/runtime lookup.
///
/// # Errors
///
/// Returns an error when shared model-routing mode parsing fails.
pub fn wendao_audio_transcript_route_config_with_model_routing_config(
    model_routing: Option<&WendaoModelRoutingTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoAttachmentRouteConfig, String> {
    wendao_audio_transcript_route_config_with_lookup(&|key| {
        model_routing_config_lookup_value(model_routing, lookup, key)
    })
}

/// Resolve image extraction route config using an injectable lookup.
///
/// # Errors
///
/// Returns an error when shared model-routing mode parsing fails.
pub fn wendao_image_extract_route_config_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoAttachmentRouteConfig, String> {
    wendao_attachment_route_config_with_lookup(
        lookup,
        WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER_ENV,
        WENDAO_IMAGE_EXTRACT_ROUTE_MODEL_ENV,
        DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_MODEL,
        WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE_ENV,
        DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE,
    )
}

/// Resolve image extraction route config from `wendao.toml` first, then env/runtime lookup.
///
/// # Errors
///
/// Returns an error when shared model-routing mode parsing fails.
pub fn wendao_image_extract_route_config_with_model_routing_config(
    model_routing: Option<&WendaoModelRoutingTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<WendaoAttachmentRouteConfig, String> {
    wendao_image_extract_route_config_with_lookup(&|key| {
        model_routing_config_lookup_value(model_routing, lookup, key)
    })
}

/// Build a Gateway-owned attachment route intent.
#[must_use]
pub fn wendao_attachment_route_intent(input: &WendaoAttachmentRouteInput) -> WendaoRouteIntent {
    WendaoRouteIntent {
        task_kind: input.task_kind.clone(),
        modality: input.modality.clone(),
        source_kind: input.source_kind.clone(),
        precision_tier: input.precision_tier.clone(),
        privacy_tier: input.privacy_tier.clone(),
        latency_budget_ms: input.latency_budget_ms,
        evidence_profile: input.evidence_profile.clone(),
        artifact_refs: input.artifact_refs.clone(),
    }
}

/// Admit one attachment request through the configured model routing mode.
///
/// # Errors
///
/// Returns an infrastructure admission error when vLLM-SR mode is active but
/// no provider hint is configured, or when deterministic policy has an empty
/// model/backend profile.
pub async fn wendao_attachment_model_route_decision(
    config: &WendaoAttachmentRouteConfig,
    input: &WendaoAttachmentRouteInput,
) -> Result<(WendaoRouteIntent, WendaoModelDecision), String> {
    let intent = wendao_attachment_route_intent(input);
    match config.model_routing_mode {
        WendaoModelRoutingMode::Deterministic => {
            let decision = deterministic_attachment_route_decision(config, input)?;
            Ok((intent, decision))
        }
        WendaoModelRoutingMode::VllmSr => {
            let provider = attachment_route_provider_hint(config)?;
            let decision = VllmSrRouteDecisionClient::new(config.vllm_sr_base_url.as_str())
                .decide(&intent, provider.as_str(), config.backend_profile.as_str())
                .await
                .map_err(|error| {
                    format!(
                        "{} attachment model route admission failed: {error}",
                        input.modality
                    )
                })?;
            Ok((intent, decision))
        }
    }
}

fn wendao_attachment_route_config_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    provider_env: &'static str,
    model_env: &'static str,
    default_model: &'static str,
    backend_profile_env: &'static str,
    default_backend_profile: &'static str,
) -> Result<WendaoAttachmentRouteConfig, String> {
    Ok(WendaoAttachmentRouteConfig {
        route_provider: optional_string_value(lookup, provider_env),
        route_model: string_value(lookup, model_env, default_model),
        backend_profile: string_value(lookup, backend_profile_env, default_backend_profile),
        model_routing_mode: wendao_model_routing_mode_with_lookup(lookup)?,
        vllm_sr_base_url: wendao_vllm_sr_base_url_with_lookup(lookup),
    })
}

fn deterministic_attachment_route_decision(
    config: &WendaoAttachmentRouteConfig,
    input: &WendaoAttachmentRouteInput,
) -> Result<WendaoModelDecision, String> {
    let selected_provider = attachment_route_provider_or_default(config);
    let selected_model = config.route_model.trim();
    if selected_model.is_empty() {
        return Err(format!(
            "{} deterministic route model must be non-empty",
            input.modality
        ));
    }
    let selected_backend_profile = config.backend_profile.trim();
    if selected_backend_profile.is_empty() {
        return Err(format!(
            "{} deterministic route backend profile must be non-empty",
            input.modality
        ));
    }
    Ok(WendaoModelDecision {
        route_id: format!(
            "deterministic:{}:{}:{}:{}",
            sanitize_route_id_part(input.task_kind.as_str()),
            sanitize_route_id_part(input.modality.as_str()),
            sanitize_route_id_part(selected_provider.as_str()),
            sanitize_route_id_part(selected_model),
        ),
        selected_provider,
        selected_model: selected_model.to_owned(),
        selected_backend_profile: selected_backend_profile.to_owned(),
        reasoning_policy: None,
        route_trace: Some("gateway deterministic attachment route policy".to_owned()),
    })
}

fn attachment_route_provider_or_default(config: &WendaoAttachmentRouteConfig) -> String {
    config
        .route_provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || DEFAULT_WENDAO_ATTACHMENT_ROUTE_PROVIDER.to_owned(),
            str::to_owned,
        )
}

fn attachment_route_provider_hint(config: &WendaoAttachmentRouteConfig) -> Result<String, String> {
    config
        .route_provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "WENDAO_MODEL_ROUTING_MODE=vllm-sr requires a route provider".to_owned())
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
