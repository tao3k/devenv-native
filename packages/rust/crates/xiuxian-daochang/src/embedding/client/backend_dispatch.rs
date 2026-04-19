use super::super::transport_http::embed_http;
#[cfg(feature = "agent-provider-litellm")]
use super::super::transport_litellm::embed_litellm;
use super::super::transport_openai::embed_openai_http;
use super::EmbeddingDispatchRuntime;

#[cfg(feature = "agent-provider-litellm")]
fn has_non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|candidate| !candidate.trim().is_empty())
}

pub(super) async fn dispatch_chunk_by_backend(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    match runtime.backend_mode {
        super::super::backend::EmbeddingBackendMode::Http => {
            dispatch_http_backend(runtime, texts, model).await
        }
        super::super::backend::EmbeddingBackendMode::OpenAiHttp => {
            dispatch_openai_backend(runtime, texts, model).await
        }
        super::super::backend::EmbeddingBackendMode::LiteLlmRs => {
            dispatch_litellm_backend(runtime, texts, model).await
        }
    }
}

async fn dispatch_http_backend(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    let primary = embed_http(&runtime.client, runtime.base_url.as_str(), texts, model).await;
    if primary.is_none() {
        tracing::debug!(
            event = "agent.embedding.http.primary_failed",
            base_url = runtime.base_url,
            model = model.unwrap_or(""),
            "embedding http primary failed"
        );
    }
    primary
}

async fn dispatch_openai_backend(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    let primary = embed_openai_http(&runtime.client, runtime.base_url.as_str(), texts, model).await;
    if primary.is_none() {
        tracing::debug!(
            event = "agent.embedding.openai_http.primary_failed",
            base_url = runtime.base_url,
            model = model.unwrap_or(""),
            "embedding openai-http primary failed"
        );
    }
    primary
}

async fn dispatch_litellm_backend(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    #[cfg(not(feature = "agent-provider-litellm"))]
    {
        return dispatch_litellm_backend_without_feature(runtime, texts, model).await;
    }
    #[cfg(feature = "agent-provider-litellm")]
    {
        return dispatch_litellm_backend_with_feature(runtime, texts, model).await;
    }
}

#[cfg(not(feature = "agent-provider-litellm"))]
async fn dispatch_litellm_backend_without_feature(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    tracing::warn!(
        event = "agent.embedding.litellm.disabled",
        "embedding backend resolved to litellm-rs but feature agent-provider-litellm is disabled; falling back to http only"
    );
    embed_http(&runtime.client, runtime.base_url.as_str(), texts, model).await
}

#[cfg(feature = "agent-provider-litellm")]
async fn dispatch_litellm_backend_with_feature(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    let has_litellm_api_key = litellm_api_key_is_present(runtime);
    let Some(model) = model else {
        tracing::warn!(
            event = "agent.embedding.litellm.missing_model",
            "embedding backend is litellm-rs but no model is configured"
        );
        return None;
    };

    if model.starts_with("ollama/") {
        return dispatch_ollama_model_with_feature(runtime, texts, model, has_litellm_api_key)
            .await;
    }

    dispatch_standard_litellm_model_with_feature(runtime, texts, model, has_litellm_api_key).await
}

#[cfg(feature = "agent-provider-litellm")]
fn litellm_api_key_is_present(runtime: &EmbeddingDispatchRuntime) -> bool {
    has_non_empty(runtime.litellm_api_key.as_deref())
}

#[cfg(feature = "agent-provider-litellm")]
async fn dispatch_ollama_model_with_feature(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: &str,
    has_litellm_api_key: bool,
) -> Option<Vec<Vec<f32>>> {
    let openai_model = model.strip_prefix("ollama/").unwrap_or(model);
    let openai_direct = embed_openai_http(
        &runtime.client,
        runtime.base_url.as_str(),
        texts,
        Some(openai_model),
    )
    .await;
    if openai_direct.is_some() {
        tracing::debug!(
            event = "agent.embedding.ollama.openai_http_direct.hit",
            model,
            openai_model,
            base_url = runtime.base_url,
            "ollama embedding served via OpenAI-compatible direct path"
        );
        return openai_direct;
    }

    tracing::debug!(
        event = "agent.embedding.ollama.openai_http_direct.miss",
        model,
        openai_model,
        base_url = runtime.base_url,
        "ollama OpenAI-compatible direct path missed; retrying /embed/batch fallback"
    );
    let http_fallback = embed_http(
        &runtime.client,
        runtime.base_url.as_str(),
        texts,
        Some(model),
    )
    .await;
    if http_fallback.is_some() {
        tracing::debug!(
            event = "agent.embedding.ollama.http_fallback.hit",
            model,
            base_url = runtime.base_url,
            "ollama embedding recovered via /embed/batch fallback"
        );
        return http_fallback;
    }

    tracing::warn!(
        event = "agent.embedding.ollama.direct_paths_failed",
        model,
        openai_model,
        base_url = runtime.base_url,
        "ollama direct embedding paths failed; trying litellm-rs provider fallback"
    );
    if !has_litellm_api_key {
        tracing::debug!(
            event = "agent.embedding.litellm.provider.skipped_missing_api_key",
            model,
            base_url = runtime.base_url,
            "litellm-rs provider fallback skipped because no API key is configured; rust-only mode disables legacy external fallback"
        );
        return None;
    }

    let litellm_fallback = embed_litellm(
        model,
        texts,
        runtime.base_url.as_str(),
        runtime.timeout_secs,
        runtime.litellm_api_key.as_deref(),
    )
    .await;
    if litellm_fallback.is_some() {
        tracing::debug!(
            event = "agent.embedding.ollama.litellm_fallback.hit",
            model,
            base_url = runtime.base_url,
            "ollama embedding recovered via litellm-rs provider fallback"
        );
        return litellm_fallback;
    }

    tracing::debug!(
        event = "agent.embedding.ollama.all_paths_failed",
        model,
        base_url = runtime.base_url,
        "all rust embedding paths failed for ollama model"
    );
    None
}

#[cfg(feature = "agent-provider-litellm")]
async fn dispatch_standard_litellm_model_with_feature(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: &str,
    has_litellm_api_key: bool,
) -> Option<Vec<Vec<f32>>> {
    if has_litellm_api_key {
        let litellm = embed_litellm(
            model,
            texts,
            runtime.base_url.as_str(),
            runtime.timeout_secs,
            runtime.litellm_api_key.as_deref(),
        )
        .await;
        if litellm.is_some() {
            return litellm;
        }
    } else {
        tracing::debug!(
            event = "agent.embedding.litellm.provider.skipped_missing_api_key",
            model,
            base_url = runtime.base_url,
            "litellm-rs provider path skipped because no API key is configured; using rust /embed/batch path only"
        );
    }

    let http_fallback = embed_http(
        &runtime.client,
        runtime.base_url.as_str(),
        texts,
        Some(model),
    )
    .await;
    if http_fallback.is_none() {
        tracing::debug!(
            event = "agent.embedding.litellm.standard_paths_failed",
            model,
            base_url = runtime.base_url,
            "provider and http fallback failed for litellm backend"
        );
    }
    http_fallback
}
