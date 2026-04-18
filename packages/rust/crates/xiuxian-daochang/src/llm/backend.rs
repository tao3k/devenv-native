use xiuxian_llm::llm::backend::{LlmBackendKind, parse_llm_backend_kind};

pub(super) type LlmBackendMode = LlmBackendKind;

pub(super) fn parse_backend_mode(raw: Option<&str>) -> LlmBackendMode {
    let trimmed = raw.map(str::trim).filter(|value| !value.is_empty());
    match parse_llm_backend_kind(trimmed) {
        Some(LlmBackendKind::OpenAiCompatibleHttp) => LlmBackendMode::OpenAiCompatibleHttp,
        Some(LlmBackendKind::LiteLlmRs) => {
            #[cfg(feature = "agent-provider-litellm")]
            {
                LlmBackendMode::LiteLlmRs
            }
            #[cfg(not(feature = "agent-provider-litellm"))]
            {
                tracing::warn!(
                    backend = %trimmed.unwrap_or("litellm_rs"),
                    "litellm-rs backend requested but agent-provider-litellm feature is disabled; using http backend"
                );
                LlmBackendMode::OpenAiCompatibleHttp
            }
        }
        None => {
            if let Some(v) = trimmed {
                tracing::warn!(
                    backend = %v,
                    "invalid XIUXIAN_DAOCHANG_LLM_BACKEND value; using default backend"
                );
            }
            #[cfg(feature = "agent-provider-litellm")]
            {
                LlmBackendMode::LiteLlmRs
            }
            #[cfg(not(feature = "agent-provider-litellm"))]
            {
                LlmBackendMode::OpenAiCompatibleHttp
            }
        }
    }
}

pub(super) fn extract_api_base_from_inference_url(inference_url: &str) -> String {
    let trimmed = inference_url.trim_end_matches('/');
    if let Some(prefix) = trimmed.strip_suffix("/chat/completions") {
        return prefix.to_string();
    }
    if let Some(prefix) = trimmed.strip_suffix("/v1/messages") {
        return prefix.to_string();
    }
    if let Some(prefix) = trimmed.strip_suffix("/messages") {
        return prefix.to_string();
    }
    trimmed.to_string()
}
