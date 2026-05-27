//! Runtime profile resolution for OpenAI-compatible multi-provider configs.

use super::client::OpenAIWireApi;
use super::error::{LlmError, LlmResult};
#[cfg(feature = "model-routing")]
use crate::model_routing::WendaoModelDecision;
use crate::resource;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// System-level Wendao LLM runtime defaults shipped by `xiuxian-llm`.
pub const WENDAO_LLM_SYSTEM_DEFAULT_TOML: &str = resource::WENDAO_LLM_SYSTEM_DEFAULT_TOML;

/// Provider-scoped runtime fields used to resolve an OpenAI-compatible profile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LlmProviderProfileInput {
    /// Provider model override.
    pub model: Option<String>,
    /// Provider base URL.
    pub base_url: Option<String>,
    /// Provider API key value or env-key reference token.
    pub api_key: Option<String>,
    /// Provider API key env name.
    pub api_key_env: Option<String>,
    /// Provider wire mode override.
    pub wire_api: Option<String>,
}

/// Aggregated `llm` config input used for runtime profile resolution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LlmRuntimeProfileInput {
    /// Flat model override.
    pub model: Option<String>,
    /// Global default model alias.
    pub default_model: Option<String>,
    /// Flat base URL override.
    pub base_url: Option<String>,
    /// Flat API key env token.
    pub api_key_env: Option<String>,
    /// Flat API key value or env-key token.
    pub api_key: Option<String>,
    /// Flat wire mode.
    pub wire_api: Option<String>,
    /// Selected default provider key.
    pub default_provider: Option<String>,
    /// Named providers by provider key.
    pub providers: HashMap<String, LlmProviderProfileInput>,
}

/// Root `[llm]` configuration from `wendao.toml`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct LlmRuntimeProfileTomlConfig {
    /// Runtime backend hint, for example `litellm`.
    #[serde(default)]
    pub backend: Option<String>,
    /// Flat model override.
    #[serde(default)]
    pub model: Option<String>,
    /// Global default model alias.
    #[serde(default)]
    pub default_model: Option<String>,
    /// Flat base URL override.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Flat API key env token.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Flat API key value or env-key token.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Flat wire mode.
    #[serde(default)]
    pub wire_api: Option<String>,
    /// Selected default provider key.
    #[serde(default)]
    pub default_provider: Option<String>,
    /// Named providers by provider key.
    #[serde(default)]
    pub providers: BTreeMap<String, LlmProviderProfileTomlConfig>,
    /// Forward-compatible extension fields.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

/// One provider entry from `[llm.providers.<name>]`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct LlmProviderProfileTomlConfig {
    /// Provider model override.
    #[serde(default)]
    pub model: Option<String>,
    /// Provider base URL.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Provider API key value or env-key reference token.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Provider API key env name.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Provider wire mode override.
    #[serde(default)]
    pub wire_api: Option<String>,
    /// Forward-compatible extension fields.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LlmRuntimeProfileTomlRoot {
    #[serde(default)]
    llm: LlmRuntimeProfileTomlConfig,
}

/// Runtime overrides and explicit env map for deterministic resolution/testing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LlmRuntimeProfileEnv {
    /// Explicit provider override.
    pub provider_override: Option<String>,
    /// Explicit model override.
    pub model_override: Option<String>,
    /// Explicit base URL override.
    pub base_url_override: Option<String>,
    /// Explicit API key override.
    pub api_key_override: Option<String>,
    /// Explicit wire mode override.
    pub wire_api_override: Option<String>,
    /// Explicit environment map; values here shadow process env.
    pub env_vars: Vec<(String, String)>,
}

/// Hard defaults for runtime profile resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmRuntimeDefaults {
    /// Fallback provider key.
    pub provider: String,
    /// Fallback model.
    pub model: String,
    /// Fallback base URL.
    pub base_url: String,
    /// Fallback API key env variable name.
    pub api_key_env: String,
    /// Fallback wire API.
    pub wire_api: OpenAIWireApi,
}

impl Default for LlmRuntimeDefaults {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            wire_api: OpenAIWireApi::ChatCompletions,
        }
    }
}

/// Parse `[llm]` from a full `wendao.toml` string.
///
/// # Errors
///
/// Returns an error when TOML parsing or deserialization fails.
pub fn llm_runtime_profile_toml_config_from_str(
    raw: &str,
) -> Result<LlmRuntimeProfileTomlConfig, String> {
    let value = toml::from_str::<toml::Value>(raw)
        .map_err(|error| format!("parse LLM runtime profile TOML: {error}"))?;
    llm_runtime_profile_toml_config_from_value(value)
}

/// Parse `[llm]` from a full `wendao.toml` value.
///
/// # Errors
///
/// Returns an error when deserialization fails.
pub fn llm_runtime_profile_toml_config_from_value(
    value: toml::Value,
) -> Result<LlmRuntimeProfileTomlConfig, String> {
    let mut config = llm_runtime_profile_system_default_config()?;
    let overlay = parse_llm_runtime_profile_without_defaults(value)?;
    merge_llm_runtime_profile_config(&mut config, overlay);
    Ok(config)
}

/// Return the source-owned Wendao LLM runtime defaults.
///
/// # Errors
///
/// Returns an error if the embedded default TOML is malformed.
pub fn llm_runtime_profile_system_default_config() -> Result<LlmRuntimeProfileTomlConfig, String> {
    let value = resource::load_llm_system_default_toml_value()?;
    parse_llm_runtime_profile_without_defaults(value)
}

fn parse_llm_runtime_profile_without_defaults(
    value: toml::Value,
) -> Result<LlmRuntimeProfileTomlConfig, String> {
    value
        .try_into::<LlmRuntimeProfileTomlRoot>()
        .map(|root| root.llm)
        .map_err(|error| format!("deserialize LLM runtime profile TOML: {error}"))
}

fn merge_llm_runtime_profile_config(
    base: &mut LlmRuntimeProfileTomlConfig,
    overlay: LlmRuntimeProfileTomlConfig,
) {
    replace_if_some(&mut base.backend, overlay.backend);
    replace_if_some(&mut base.model, overlay.model);
    replace_if_some(&mut base.default_model, overlay.default_model);
    replace_if_some(&mut base.base_url, overlay.base_url);
    replace_if_some(&mut base.api_key_env, overlay.api_key_env);
    replace_if_some(&mut base.api_key, overlay.api_key);
    replace_if_some(&mut base.wire_api, overlay.wire_api);
    replace_if_some(&mut base.default_provider, overlay.default_provider);
    for (provider_name, provider) in overlay.providers {
        base.providers
            .entry(provider_name)
            .and_modify(|existing| merge_llm_provider_profile_config(existing, provider.clone()))
            .or_insert(provider);
    }
    base.extra.extend(overlay.extra);
}

fn merge_llm_provider_profile_config(
    base: &mut LlmProviderProfileTomlConfig,
    overlay: LlmProviderProfileTomlConfig,
) {
    replace_if_some(&mut base.model, overlay.model);
    replace_if_some(&mut base.base_url, overlay.base_url);
    replace_if_some(&mut base.api_key, overlay.api_key);
    replace_if_some(&mut base.api_key_env, overlay.api_key_env);
    replace_if_some(&mut base.wire_api, overlay.wire_api);
    base.extra.extend(overlay.extra);
}

fn replace_if_some<T>(slot: &mut Option<T>, value: Option<T>) {
    if value.is_some() {
        *slot = value;
    }
}

/// Convert a TOML LLM runtime profile into the resolver input shape.
#[must_use]
pub fn llm_runtime_profile_input_from_toml_config(
    config: LlmRuntimeProfileTomlConfig,
) -> LlmRuntimeProfileInput {
    let providers = config
        .providers
        .into_iter()
        .map(|(name, provider)| {
            (
                name,
                LlmProviderProfileInput {
                    model: provider.model,
                    base_url: provider.base_url,
                    api_key: provider.api_key,
                    api_key_env: provider.api_key_env,
                    wire_api: provider.wire_api,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    LlmRuntimeProfileInput {
        model: config.model,
        default_model: config.default_model,
        base_url: config.base_url,
        api_key_env: config.api_key_env,
        api_key: config.api_key,
        wire_api: config.wire_api,
        default_provider: config.default_provider,
        providers,
    }
}

/// Fully resolved OpenAI-compatible runtime profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLlmRuntimeProfile {
    /// Resolved provider key.
    pub provider_name: String,
    /// Resolved model.
    pub model: String,
    /// Resolved base URL.
    pub base_url: String,
    /// Resolved API key env name.
    pub api_key_env: String,
    /// Resolved API key value.
    pub api_key: String,
    /// Resolved wire API.
    pub wire_api: OpenAIWireApi,
}

/// Resolve one OpenAI-compatible runtime profile from flat + provider-scoped inputs.
///
/// # Errors
///
/// Returns an error when required values cannot be resolved (for example API key or base URL).
pub fn resolve_openai_runtime_profile(
    input: &LlmRuntimeProfileInput,
    env: &LlmRuntimeProfileEnv,
    defaults: &LlmRuntimeDefaults,
) -> LlmResult<ResolvedLlmRuntimeProfile> {
    let provider_name = first_non_empty([
        env.provider_override.as_deref(),
        input.default_provider.as_deref(),
        Some(defaults.provider.as_str()),
    ])
    .ok_or_else(|| LlmError::Internal {
        message: "missing LLM provider name".to_string(),
    })?;

    let provider_cfg = find_provider_case_insensitive(&input.providers, provider_name.as_str());

    let model = first_non_empty([
        env.model_override.as_deref(),
        provider_cfg.and_then(|cfg| cfg.model.as_deref()),
        input.model.as_deref(),
        input.default_model.as_deref(),
        Some(defaults.model.as_str()),
    ])
    .ok_or_else(|| LlmError::Internal {
        message: "missing LLM model".to_string(),
    })?;

    let base_url = first_non_empty([
        env.base_url_override.as_deref(),
        provider_cfg.and_then(|cfg| cfg.base_url.as_deref()),
        input.base_url.as_deref(),
        Some(defaults.base_url.as_str()),
    ])
    .ok_or_else(|| LlmError::Internal {
        message: "missing LLM base URL".to_string(),
    })?;

    let wire_api = OpenAIWireApi::parse(
        first_non_empty([
            env.wire_api_override.as_deref(),
            provider_cfg.and_then(|cfg| cfg.wire_api.as_deref()),
            input.wire_api.as_deref(),
            Some(defaults.wire_api.as_str()),
        ])
        .as_deref(),
    );

    let default_api_key_env = first_non_empty([Some(defaults.api_key_env.as_str())])
        .unwrap_or_else(|| "OPENAI_API_KEY".to_string());

    let (api_key, api_key_env) = resolve_api_key(
        provider_cfg.and_then(|cfg| cfg.api_key.as_deref()),
        provider_cfg.and_then(|cfg| cfg.api_key_env.as_deref()),
        input.api_key.as_deref(),
        input.api_key_env.as_deref(),
        env,
        default_api_key_env.as_str(),
    )?;

    Ok(ResolvedLlmRuntimeProfile {
        provider_name,
        model,
        base_url,
        api_key_env,
        api_key,
        wire_api,
    })
}

/// Return runtime overrides with a Gateway-selected model route decision.
///
/// Existing explicit API-key, base-URL, wire-mode, and test environment values
/// are preserved. The model decision supplies provider and model overrides so
/// the normal provider profile resolver still owns endpoint and credential
/// lookup.
#[cfg(feature = "model-routing")]
#[must_use]
pub fn runtime_profile_env_with_model_decision(
    env: &LlmRuntimeProfileEnv,
    decision: &WendaoModelDecision,
) -> LlmRuntimeProfileEnv {
    let mut routed = env.clone();
    routed.provider_override = Some(decision.selected_provider.clone());
    routed.model_override = Some(decision.selected_model.clone());
    routed
}

fn resolve_api_key(
    provider_api_key: Option<&str>,
    provider_api_key_env: Option<&str>,
    flat_api_key: Option<&str>,
    flat_api_key_env: Option<&str>,
    env: &LlmRuntimeProfileEnv,
    default_api_key_env: &str,
) -> LlmResult<(String, String)> {
    let key_selector = first_non_empty([
        provider_api_key_env,
        provider_api_key,
        flat_api_key_env,
        flat_api_key,
        Some(default_api_key_env),
    ])
    .ok_or_else(|| LlmError::Internal {
        message: "missing LLM API key selector".to_string(),
    })?;

    let configured_api_key_env = parse_env_reference(key_selector.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            if is_env_key_token(key_selector.as_str()) {
                Some(key_selector.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| default_api_key_env.to_string());

    if let Some(explicit_key) = normalize_non_empty(env.api_key_override.as_deref()) {
        return Ok((explicit_key, configured_api_key_env));
    }

    if let Some(env_name) = parse_env_reference(key_selector.as_str()) {
        return resolve_env_key_value(env, env_name).ok_or_else(|| LlmError::Internal {
            message: format!("missing LLM API key in environment variable `{env_name}`"),
        });
    }

    if is_env_key_token(key_selector.as_str()) {
        return resolve_env_key_value(env, key_selector.as_str()).ok_or_else(|| {
            LlmError::Internal {
                message: format!("missing LLM API key in environment variable `{key_selector}`"),
            }
        });
    }

    Ok((key_selector, configured_api_key_env))
}

fn resolve_env_key_value(env: &LlmRuntimeProfileEnv, env_key: &str) -> Option<(String, String)> {
    resolve_env_value(env, env_key).map(|value| (value, env_key.to_string()))
}

fn resolve_env_value(env: &LlmRuntimeProfileEnv, key: &str) -> Option<String> {
    match override_env_state(env, key) {
        EnvOverrideState::Value(value) => Some(value),
        EnvOverrideState::Empty => None,
        EnvOverrideState::Missing => std::env::var(key)
            .ok()
            .and_then(|value| normalize_non_empty(Some(value.as_str()))),
    }
}

fn find_provider_case_insensitive<'a>(
    providers: &'a HashMap<String, LlmProviderProfileInput>,
    provider_name: &str,
) -> Option<&'a LlmProviderProfileInput> {
    providers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(provider_name))
        .map(|(_, cfg)| cfg)
}

fn parse_env_reference(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("env:") {
        let value = rest.trim();
        return if value.is_empty() { None } else { Some(value) };
    }
    if let Some(rest) = trimmed.strip_prefix("${")
        && let Some(env_name) = rest.strip_suffix('}')
    {
        let value = env_name.trim();
        return if value.is_empty() { None } else { Some(value) };
    }
    None
}

fn first_non_empty<'a>(candidates: impl IntoIterator<Item = Option<&'a str>>) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .find_map(|value| normalize_non_empty(Some(value)))
}

fn normalize_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToString::to_string)
}

fn is_env_key_token(raw: &str) -> bool {
    let trimmed = raw.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnvOverrideState {
    Missing,
    Empty,
    Value(String),
}

fn override_env_state(env: &LlmRuntimeProfileEnv, key: &str) -> EnvOverrideState {
    let Some((_, value)) = env.env_vars.iter().find(|(candidate, _)| candidate == key) else {
        return EnvOverrideState::Missing;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        EnvOverrideState::Empty
    } else {
        EnvOverrideState::Value(trimmed.to_string())
    }
}
