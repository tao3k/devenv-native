//! TOML-backed model routing configuration.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::resource;

use super::{
    WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE_ENV, WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL_ENV,
    WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER_ENV, WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV,
    WENDAO_CHAT_ROUTE_MODEL_ENV, WENDAO_CHAT_ROUTE_PROVIDER_ENV,
    WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE_ENV, WENDAO_IMAGE_EXTRACT_ROUTE_MODEL_ENV,
    WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER_ENV, WENDAO_MODEL_ROUTING_MODE_ENV,
    WENDAO_VLLM_SR_BASE_URL_ENV,
};

/// System-level Wendao model-routing defaults shipped by `xiuxian-llm`.
pub const WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML: &str =
    resource::WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML;

/// Root `[model_routing]` configuration from `wendao.toml`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WendaoModelRoutingTomlConfig {
    /// Routing mode, for example `deterministic` or `vllm-sr`.
    #[serde(default)]
    pub mode: Option<String>,
    /// vLLM-SR base URL used when vLLM-SR mode is enabled.
    #[serde(default)]
    pub vllm_sr_base_url: Option<String>,
    /// Default provider for route entries without a specific provider.
    #[serde(default)]
    pub default_provider: Option<String>,
    /// Chat route configuration.
    #[serde(default)]
    pub chat: WendaoRouteTomlConfig,
    /// Audio transcript route configuration.
    #[serde(default)]
    pub audio_transcript: WendaoRouteTomlConfig,
    /// Image extraction route configuration.
    #[serde(default)]
    pub image_extract: WendaoRouteTomlConfig,
    /// Forward-compatible extension fields.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

/// One model-backed route entry from `[model_routing.*]`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WendaoRouteTomlConfig {
    /// Provider key selected by Gateway policy.
    #[serde(default)]
    pub provider: Option<String>,
    /// Provider model id selected by deterministic policy.
    #[serde(default)]
    pub model: Option<String>,
    /// Wendao backend profile selected for execution.
    #[serde(default)]
    pub backend_profile: Option<String>,
    /// Forward-compatible extension fields.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoModelRoutingTomlRoot {
    #[serde(default)]
    model_routing: WendaoModelRoutingTomlConfig,
}

static MODEL_ROUTING_SYSTEM_DEFAULT_CONFIG: OnceLock<Result<WendaoModelRoutingTomlConfig, String>> =
    OnceLock::new();

/// Parse `[model_routing]` from a full `wendao.toml` string.
///
/// # Errors
///
/// Returns an error when TOML parsing or deserialization fails.
pub fn wendao_model_routing_config_from_toml_str(
    raw: &str,
) -> Result<WendaoModelRoutingTomlConfig, String> {
    let value = toml::from_str::<toml::Value>(raw)
        .map_err(|error| format!("parse Wendao model routing TOML: {error}"))?;
    wendao_model_routing_config_from_toml_value(value)
}

/// Parse `[model_routing]` from a full `wendao.toml` value.
///
/// # Errors
///
/// Returns an error when deserialization fails.
pub fn wendao_model_routing_config_from_toml_value(
    value: toml::Value,
) -> Result<WendaoModelRoutingTomlConfig, String> {
    parse_model_routing_config_without_defaults(value)
}

/// Return the source-owned Wendao model-routing defaults.
///
/// # Errors
///
/// Returns an error if the embedded default TOML is malformed.
pub fn wendao_model_routing_system_default_config() -> Result<WendaoModelRoutingTomlConfig, String>
{
    cached_model_routing_system_default_config().cloned()
}

fn cached_model_routing_system_default_config()
-> Result<&'static WendaoModelRoutingTomlConfig, String> {
    match MODEL_ROUTING_SYSTEM_DEFAULT_CONFIG.get_or_init(parse_model_routing_system_default_config)
    {
        Ok(config) => Ok(config),
        Err(error) => Err(error.clone()),
    }
}

fn parse_model_routing_system_default_config() -> Result<WendaoModelRoutingTomlConfig, String> {
    let value = resource::load_model_routing_system_default_toml_value()?;
    parse_model_routing_config_without_defaults(value)
}

fn parse_model_routing_config_without_defaults(
    value: toml::Value,
) -> Result<WendaoModelRoutingTomlConfig, String> {
    value
        .try_into::<WendaoModelRoutingTomlRoot>()
        .map(|root| root.model_routing)
        .map_err(|error| format!("deserialize Wendao model routing TOML: {error}"))
}

pub(crate) fn model_routing_config_lookup_value(
    config: Option<&WendaoModelRoutingTomlConfig>,
    env_lookup: &dyn Fn(&str) -> Option<String>,
    key: &str,
) -> Option<String> {
    if let Some(value) = toml_value_for_env_key(config, key) {
        return Some(value);
    }
    if let Some(value) = env_lookup(key) {
        return Some(value);
    }
    if let Ok(default_config) = cached_model_routing_system_default_config()
        && let Some(value) = toml_value_for_env_key(Some(default_config), key)
    {
        return Some(value);
    }
    None
}

fn toml_value_for_env_key(
    config: Option<&WendaoModelRoutingTomlConfig>,
    key: &str,
) -> Option<String> {
    let config = config?;
    let raw = match key {
        WENDAO_MODEL_ROUTING_MODE_ENV => config.mode.as_deref(),
        WENDAO_VLLM_SR_BASE_URL_ENV => config.vllm_sr_base_url.as_deref(),
        WENDAO_CHAT_ROUTE_PROVIDER_ENV => route_provider(&config.chat, config),
        WENDAO_CHAT_ROUTE_MODEL_ENV => config.chat.model.as_deref(),
        WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV => config.chat.backend_profile.as_deref(),
        WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER_ENV => {
            route_provider(&config.audio_transcript, config)
        }
        WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL_ENV => config.audio_transcript.model.as_deref(),
        WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE_ENV => {
            config.audio_transcript.backend_profile.as_deref()
        }
        WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER_ENV => route_provider(&config.image_extract, config),
        WENDAO_IMAGE_EXTRACT_ROUTE_MODEL_ENV => config.image_extract.model.as_deref(),
        WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE_ENV => {
            config.image_extract.backend_profile.as_deref()
        }
        _ => None,
    }?;
    normalized_non_empty(raw)
}

fn route_provider<'a>(
    route: &'a WendaoRouteTomlConfig,
    config: &'a WendaoModelRoutingTomlConfig,
) -> Option<&'a str> {
    route
        .provider
        .as_deref()
        .or(config.default_provider.as_deref())
}

fn normalized_non_empty(value: &str) -> Option<String> {
    let normalized = value.trim();
    (!normalized.is_empty()).then(|| normalized.to_owned())
}
