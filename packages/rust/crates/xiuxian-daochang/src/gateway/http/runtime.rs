use std::sync::Arc;

use anyhow::Result;
use axum::http::StatusCode;
use xiuxian_llm::embedding::backend::{EmbeddingBackendKind, parse_embedding_backend_kind};
use xiuxian_macros::env_non_empty;

use crate::config::{RuntimeSettings, load_runtime_settings};
use crate::embedding::EmbeddingClient;

use super::types::GatewayEmbeddingRuntime;

const DEFAULT_EMBED_TIMEOUT_SECS: u64 = 15;
const DEFAULT_EMBED_UPSTREAM_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_FALLBACK_EMBED_DIM: usize = 1024;
const GATEWAY_ALLOW_INPROC_EMBED_ENV: &str = "XIUXIAN_DAOCHANG_GATEWAY_ALLOW_INPROC_EMBED";

pub(super) fn trim_non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn apply_gateway_embedding_memory_guard(runtime_settings: &RuntimeSettings) -> RuntimeSettings {
    apply_gateway_embedding_memory_guard_with_inputs(
        runtime_settings,
        env_non_empty!("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_BACKEND").as_deref(),
        env_non_empty!("XIUXIAN_DAOCHANG_EMBED_BACKEND").as_deref(),
        env_non_empty!(GATEWAY_ALLOW_INPROC_EMBED_ENV).as_deref(),
    )
}

fn apply_gateway_embedding_memory_guard_with_inputs(
    runtime_settings: &RuntimeSettings,
    env_memory_backend: Option<&str>,
    env_embed_backend: Option<&str>,
    allow_inproc_embed_raw: Option<&str>,
) -> RuntimeSettings {
    let mut guarded = runtime_settings.clone();

    if let Some(backend) = trim_non_empty(env_memory_backend) {
        guarded.memory.embedding_backend = Some(backend);
    } else if let Some(backend) = trim_non_empty(env_embed_backend) {
        guarded.memory.embedding_backend = Some(backend);
    }

    let allow_inproc_embed = allow_inproc_embed_raw
        .map(|raw| raw.trim().to_ascii_lowercase())
        .is_some_and(|raw| matches!(raw.as_str(), "1" | "true" | "yes" | "on"));
    if allow_inproc_embed {
        return guarded;
    }

    guarded
}

pub(crate) fn apply_gateway_embedding_memory_guard_for_tests(
    runtime_settings: &RuntimeSettings,
    env_memory_backend: Option<&str>,
    env_embed_backend: Option<&str>,
    allow_inproc_embed_raw: Option<&str>,
) -> RuntimeSettings {
    apply_gateway_embedding_memory_guard_with_inputs(
        runtime_settings,
        env_memory_backend,
        env_embed_backend,
        allow_inproc_embed_raw,
    )
}

pub(crate) fn resolve_embed_model(
    requested_model: Option<&str>,
    default_model: Option<&str>,
) -> Result<String, (StatusCode, String)> {
    let configured = trim_non_empty(default_model);
    let requested = trim_non_empty(requested_model);

    if let Some(configured_model) = configured {
        if let Some(requested_model) = requested.as_deref()
            && requested_model != configured_model
        {
            tracing::debug!(
                event = "gateway.embedding.model_override_ignored",
                requested_model,
                configured_model,
                "ignoring request model override; using configured embedding model"
            );
        }
        return Ok(configured_model);
    }

    requested.ok_or((
        StatusCode::BAD_REQUEST,
        "embedding model must be provided (request.model or embedding.model)".to_string(),
    ))
}

pub(crate) fn resolve_embed_base_url(
    runtime_settings: &RuntimeSettings,
    backend_hint: Option<&str>,
) -> String {
    let memory_base_url = trim_non_empty(runtime_settings.memory.embedding_base_url.as_deref());
    let litellm_api_base = trim_non_empty(runtime_settings.embedding.litellm_api_base.as_deref());
    let embedding_client_url = trim_non_empty(runtime_settings.embedding.client_url.as_deref());
    let backend_mode = parse_embedding_backend_kind(
        backend_hint
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    );
    let selected = match backend_mode {
        Some(EmbeddingBackendKind::LiteLlmRs | EmbeddingBackendKind::OpenAiHttp) => {
            litellm_api_base
                .or(memory_base_url)
                .or(embedding_client_url)
        }
        _ => memory_base_url
            .or(embedding_client_url)
            .or(litellm_api_base),
    };
    selected.unwrap_or_else(|| DEFAULT_EMBED_UPSTREAM_BASE_URL.to_string())
}

pub(super) fn build_embedding_runtime() -> GatewayEmbeddingRuntime {
    let runtime_settings = load_runtime_settings();
    let guarded_runtime_settings = apply_gateway_embedding_memory_guard(&runtime_settings);
    build_embedding_runtime_from_settings(&guarded_runtime_settings)
}

pub(super) async fn build_embedding_runtime_for_gateway() -> Result<GatewayEmbeddingRuntime> {
    let runtime_settings = load_runtime_settings();
    let guarded_runtime_settings = apply_gateway_embedding_memory_guard(&runtime_settings);
    Ok(build_embedding_runtime_for_settings(
        &guarded_runtime_settings,
    ))
}

pub(crate) fn build_embedding_runtime_for_settings(
    runtime_settings: &RuntimeSettings,
) -> GatewayEmbeddingRuntime {
    build_embedding_runtime_from_settings(runtime_settings)
}

fn build_embedding_runtime_from_settings(
    runtime_settings: &RuntimeSettings,
) -> GatewayEmbeddingRuntime {
    let backend_hint = resolve_backend_hint(runtime_settings);
    let base_url = resolve_runtime_embed_base_url(runtime_settings, backend_hint.as_deref(), None);
    let timeout_secs = runtime_settings
        .embedding
        .timeout_secs
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_EMBED_TIMEOUT_SECS);
    let batch_max_size = runtime_settings
        .embedding
        .batch_max_size
        .filter(|value| *value > 0);
    let batch_max_concurrency = runtime_settings
        .embedding
        .batch_max_concurrency
        .filter(|value| *value > 0);
    let default_model = trim_non_empty(runtime_settings.memory.embedding_model.as_deref())
        .or_else(|| trim_non_empty(runtime_settings.embedding.litellm_model.as_deref()))
        .or_else(|| trim_non_empty(runtime_settings.embedding.model.as_deref()));
    let fallback_embedding_dim = runtime_settings
        .memory
        .embedding_dim
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_FALLBACK_EMBED_DIM);

    tracing::info!(
        event = "gateway.embedding.runtime.initialized",
        backend = backend_hint.as_deref().unwrap_or("default"),
        base_url = %base_url,
        timeout_secs,
        has_default_model = default_model.is_some(),
        fallback_embedding_dim,
        batch_max_size = ?batch_max_size,
        batch_max_concurrency = ?batch_max_concurrency,
        "gateway embedding runtime initialized"
    );

    let client = EmbeddingClient::new_with_backend_and_tuning(
        &base_url,
        timeout_secs,
        backend_hint.as_deref(),
        batch_max_size,
        batch_max_concurrency,
    );

    GatewayEmbeddingRuntime {
        client: Arc::new(client),
        default_model,
        fallback_embedding_dim,
    }
}

pub(crate) fn resolve_runtime_embed_base_url(
    runtime_settings: &RuntimeSettings,
    backend_hint: Option<&str>,
    base_url_override: Option<&str>,
) -> String {
    if let Some(override_url) = trim_non_empty(base_url_override) {
        return override_url;
    }
    resolve_embed_base_url(runtime_settings, backend_hint)
}

fn resolve_backend_hint(runtime_settings: &RuntimeSettings) -> Option<String> {
    trim_non_empty(runtime_settings.memory.embedding_backend.as_deref())
        .or_else(|| trim_non_empty(runtime_settings.embedding.backend.as_deref()))
}
