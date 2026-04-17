use crate::config::RuntimeSettings;
use xiuxian_macros::env_non_empty;

use super::{DEFAULT_ANTHROPIC_KEY_ENV, DEFAULT_MINIMAX_KEY_ENV, DEFAULT_OPENAI_KEY_ENV};

const DEFAULT_MINIMAX_API_BASE: &str = "https://api.minimax.io/v1";
const DEFAULT_MINIMAX_MODEL: &str = "MiniMax-M2.5";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::llm) enum LiteLlmProviderMode {
    OpenAi,
    Minimax,
    Anthropic,
}

impl LiteLlmProviderMode {
    pub(in crate::llm) fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Minimax => "minimax",
            Self::Anthropic => "anthropic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::llm) enum LiteLlmWireApi {
    ChatCompletions,
    Responses,
}

impl LiteLlmWireApi {
    pub(in crate::llm) const fn as_str(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat_completions",
            Self::Responses => "responses",
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::llm) struct ProviderSettings {
    pub mode: LiteLlmProviderMode,
    pub wire_api: LiteLlmWireApi,
    pub source: &'static str,
    pub api_key: Option<String>,
    pub api_key_env: String,
    pub minimax_api_base: String,
    pub model: String,
    pub timeout_secs: u64,
    pub max_tokens: Option<u32>,
    pub max_in_flight: Option<usize>,
}

fn parse_litellm_provider_mode(raw: Option<&str>) -> LiteLlmProviderMode {
    match raw.map(str::trim).map(str::to_ascii_lowercase) {
        Some(value) if value == "minimax" => LiteLlmProviderMode::Minimax,
        Some(value) if value == "anthropic" => LiteLlmProviderMode::Anthropic,
        Some(value)
            if matches!(
                value.as_str(),
                "openai"
                    | "openai_like"
                    | "openai-like"
                    | "openai_compatible"
                    | "openai-compatible"
                    | "deepseek"
            ) =>
        {
            LiteLlmProviderMode::OpenAi
        }
        Some(value) if value.is_empty() => LiteLlmProviderMode::OpenAi,
        None => LiteLlmProviderMode::OpenAi,
        Some(value) => {
            tracing::warn!(
                provider = %value,
                "unsupported litellm provider; using openai provider mode"
            );
            LiteLlmProviderMode::OpenAi
        }
    }
}

fn parse_litellm_wire_api(raw: Option<&str>) -> LiteLlmWireApi {
    match raw.map(str::trim).map(str::to_ascii_lowercase) {
        Some(value)
            if matches!(
                value.as_str(),
                "responses" | "response" | "openai_responses"
            ) =>
        {
            LiteLlmWireApi::Responses
        }
        Some(value)
            if matches!(
                value.as_str(),
                "" | "chat" | "chat_completions" | "chat-completions"
            ) =>
        {
            LiteLlmWireApi::ChatCompletions
        }
        None => LiteLlmWireApi::ChatCompletions,
        Some(value) => {
            tracing::warn!(
                wire_api = %value,
                "unsupported llm wire_api; using chat_completions"
            );
            LiteLlmWireApi::ChatCompletions
        }
    }
}

fn default_api_key_env(mode: LiteLlmProviderMode) -> &'static str {
    match mode {
        LiteLlmProviderMode::OpenAi => DEFAULT_OPENAI_KEY_ENV,
        LiteLlmProviderMode::Minimax => DEFAULT_MINIMAX_KEY_ENV,
        LiteLlmProviderMode::Anthropic => DEFAULT_ANTHROPIC_KEY_ENV,
    }
}

fn normalize_provider_model(mode: LiteLlmProviderMode, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    match mode {
        LiteLlmProviderMode::Minimax => normalize_minimax_model(trimmed),
        _ => trimmed.to_string(),
    }
}

fn normalize_minimax_model(raw: &str) -> String {
    let stripped = raw
        .strip_prefix("minimax/")
        .or_else(|| raw.strip_prefix("minimax:"))
        .unwrap_or(raw);
    let lower = stripped.to_ascii_lowercase();

    match lower.as_str() {
        "minimax-m2.1-highspeed" => "MiniMax-M2.1-lightning".to_string(),
        "minimax-m2.5-highspeed" => "MiniMax-M2.5-lightning".to_string(),
        "minimax-m2.1" => "MiniMax-M2.1".to_string(),
        "minimax-m2.5" => "MiniMax-M2.5".to_string(),
        _ => {
            if stripped.starts_with("MiniMax-") {
                return stripped.to_string();
            }
            if let Some(suffix) = lower.strip_prefix("minimax-") {
                return format!("MiniMax-{suffix}");
            }
            stripped.to_string()
        }
    }
}

fn is_env_var_name(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

fn resolve_api_key_reference(
    configured: Option<&str>,
    fallback_env: &str,
) -> (Option<String>, String) {
    let Some(raw) = configured.map(str::trim).filter(|value| !value.is_empty()) else {
        return (None, fallback_env.to_string());
    };
    if let Some(env_name) = raw.strip_prefix("env:")
        && is_env_var_name(env_name)
    {
        return (None, env_name.to_string());
    }
    if raw.starts_with("${")
        && raw.ends_with('}')
        && raw.len() > 3
        && is_env_var_name(&raw[2..raw.len() - 1])
    {
        return (None, raw[2..raw.len() - 1].to_string());
    }
    if is_env_var_name(raw) {
        return (None, raw.to_string());
    }
    (Some(raw.to_string()), fallback_env.to_string())
}

pub(in crate::llm) fn resolve_provider_settings(
    runtime_settings: &RuntimeSettings,
    requested_model: String,
) -> ProviderSettings {
    let env_provider = env_non_empty!("OMNI_AGENT_LLM_PROVIDER");
    resolve_provider_settings_with_env(
        runtime_settings,
        requested_model,
        env_provider.as_deref(),
        env_non_empty!("MINIMAX_API_BASE").as_deref(),
    )
}

pub(in crate::llm) fn resolve_provider_settings_with_env(
    runtime_settings: &RuntimeSettings,
    requested_model: String,
    env_provider_raw: Option<&str>,
    env_minimax_api_base_raw: Option<&str>,
) -> ProviderSettings {
    let env_provider = env_provider_raw
        .map(str::trim)
        .filter(|raw| !raw.is_empty());
    let (mode, source) = if let Some(raw) = env_provider {
        (parse_litellm_provider_mode(Some(raw)), "env")
    } else {
        let settings_provider = runtime_settings
            .inference
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(raw) = settings_provider {
            (parse_litellm_provider_mode(Some(raw)), "settings")
        } else {
            (LiteLlmProviderMode::OpenAi, "default")
        }
    };
    let wire_api = parse_litellm_wire_api(
        env_non_empty!("OMNI_AGENT_LLM_WIRE_API")
            .as_deref()
            .or(runtime_settings.inference.wire_api.as_deref()),
    );

    let settings_model = runtime_settings
        .inference
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let raw_model = if requested_model.trim().is_empty() {
        settings_model.unwrap_or(requested_model)
    } else {
        requested_model
    };
    let model = if raw_model.trim().is_empty() && mode == LiteLlmProviderMode::Minimax {
        DEFAULT_MINIMAX_MODEL.to_string()
    } else {
        normalize_provider_model(mode, raw_model.as_str())
    };
    let (api_key, api_key_env) = resolve_api_key_reference(
        runtime_settings.inference.api_key.as_deref(),
        default_api_key_env(mode),
    );
    let minimax_api_base = env_minimax_api_base_raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            runtime_settings
                .inference
                .base_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| DEFAULT_MINIMAX_API_BASE.to_string());
    let timeout_secs = runtime_settings
        .inference
        .timeout
        .filter(|value| *value > 0)
        .unwrap_or(60);
    let max_tokens = runtime_settings
        .inference
        .max_tokens
        .filter(|value| *value > 0)
        .map(|value| u32::try_from(value.min(u64::from(u32::MAX))).unwrap_or(u32::MAX));
    let max_in_flight = runtime_settings
        .inference
        .max_in_flight
        .filter(|value| *value > 0);

    ProviderSettings {
        mode,
        wire_api,
        source,
        api_key,
        api_key_env,
        minimax_api_base,
        model,
        timeout_secs,
        max_tokens,
        max_in_flight,
    }
}
