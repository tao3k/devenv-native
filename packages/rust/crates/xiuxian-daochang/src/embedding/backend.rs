use crate::config::load_runtime_settings;
use xiuxian_llm::embedding::backend::{EmbeddingBackendKind, parse_embedding_backend_kind};
use xiuxian_macros::env_non_empty;

const DEFAULT_EMBED_TIMEOUT_SECS: u64 = 15;
const MIN_EMBED_TIMEOUT_SECS: u64 = 1;
const MAX_EMBED_TIMEOUT_SECS: u64 = 300;
const MAX_EMBED_MAX_IN_FLIGHT: usize = 4096;

pub(crate) type EmbeddingBackendMode = EmbeddingBackendKind;

#[derive(Debug, Clone)]
pub(crate) struct EmbeddingBackendSettings {
    pub(crate) mode: EmbeddingBackendMode,
    pub(crate) source: &'static str,
    pub(crate) timeout_secs: u64,
    pub(crate) max_in_flight: Option<usize>,
    pub(crate) default_model: Option<String>,
}

pub(crate) fn resolve_backend_settings(
    default_timeout_secs: u64,
    backend_hint: Option<&str>,
) -> EmbeddingBackendSettings {
    let runtime_settings = load_runtime_settings();
    let hint_backend = backend_hint
        .map(str::trim)
        .map(ToString::to_string)
        .filter(|raw| !raw.is_empty());
    let env_backend = env_non_empty!("XIUXIAN_DAOCHANG_EMBED_BACKEND");
    let settings_backend = runtime_settings
        .embedding
        .backend
        .as_deref()
        .map(str::trim)
        .map(ToString::to_string)
        .filter(|raw| !raw.is_empty());
    let (mode, source) = if let Some(raw) = hint_backend.as_deref() {
        (parse_backend_mode(Some(raw)), "memory_config")
    } else if let Some(raw) = env_backend.as_deref() {
        (parse_backend_mode(Some(raw)), "env")
    } else if let Some(raw) = settings_backend.as_deref() {
        (parse_backend_mode(Some(raw)), "settings")
    } else if let Some(raw) = env_non_empty!("XIUXIAN_DAOCHANG_LLM_BACKEND") {
        (parse_backend_mode(Some(raw.as_str())), "llm_env")
    } else {
        (default_backend_mode(), "default")
    };

    let timeout_secs = env_non_empty!("XIUXIAN_DAOCHANG_EMBED_TIMEOUT_SECS")
        .and_then(|raw| raw.parse::<u64>().ok())
        .or(runtime_settings.embedding.timeout_secs)
        .unwrap_or(default_timeout_secs.max(DEFAULT_EMBED_TIMEOUT_SECS))
        .clamp(MIN_EMBED_TIMEOUT_SECS, MAX_EMBED_TIMEOUT_SECS);

    let max_in_flight = env_non_empty!("XIUXIAN_DAOCHANG_EMBED_MAX_IN_FLIGHT")
        .and_then(|raw| raw.parse::<usize>().ok())
        .or(runtime_settings
            .embedding
            .max_in_flight
            .filter(|value| *value > 0))
        .map(|value| value.min(MAX_EMBED_MAX_IN_FLIGHT));

    let default_model = env_non_empty!("XIUXIAN_DAOCHANG_EMBED_MODEL").or_else(|| {
        runtime_settings
            .embedding
            .litellm_model
            .as_deref()
            .map(str::trim)
            .map(ToString::to_string)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                runtime_settings
                    .embedding
                    .model
                    .as_deref()
                    .map(str::trim)
                    .map(ToString::to_string)
                    .filter(|value| !value.is_empty())
            })
    });

    EmbeddingBackendSettings {
        mode,
        source,
        timeout_secs,
        max_in_flight,
        default_model,
    }
}

fn parse_backend_mode(raw: Option<&str>) -> EmbeddingBackendMode {
    let trimmed = raw.map(str::trim).filter(|value| !value.is_empty());
    match parse_embedding_backend_kind(trimmed) {
        Some(EmbeddingBackendKind::Http) => EmbeddingBackendMode::Http,
        Some(EmbeddingBackendKind::OpenAiHttp) => EmbeddingBackendMode::OpenAiHttp,
        Some(EmbeddingBackendKind::LiteLlmRs) => {
            #[cfg(feature = "agent-provider-litellm")]
            {
                EmbeddingBackendMode::LiteLlmRs
            }
            #[cfg(not(feature = "agent-provider-litellm"))]
            {
                tracing::warn!(
                    requested_backend = %trimmed.unwrap_or("litellm_rs"),
                    "litellm-rs embedding backend requested but feature agent-provider-litellm is disabled; using http backend"
                );
                EmbeddingBackendMode::Http
            }
        }
        None => {
            if let Some(value) = trimmed {
                let fallback = default_backend_mode();
                tracing::warn!(
                    invalid_value = %value,
                    fallback_backend = fallback.as_str(),
                    "invalid embedding backend; defaulting to runtime backend"
                );
            }
            default_backend_mode()
        }
    }
}

pub(crate) fn test_parse_backend_mode(raw: Option<&str>) -> EmbeddingBackendMode {
    parse_backend_mode(raw)
}

fn default_backend_mode() -> EmbeddingBackendMode {
    #[cfg(feature = "agent-provider-litellm")]
    {
        EmbeddingBackendMode::LiteLlmRs
    }
    #[cfg(not(feature = "agent-provider-litellm"))]
    {
        EmbeddingBackendMode::Http
    }
}
