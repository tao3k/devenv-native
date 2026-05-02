//! Test-facing embedding helper surface.

use super::{backend, transport_http};

pub(crate) fn test_parse_backend_mode(
    raw: Option<&str>,
) -> xiuxian_llm::embedding::EmbeddingBackendKind {
    backend::test_parse_backend_mode(raw)
}

#[cfg(feature = "agent-provider-litellm")]
pub(crate) const TEST_OLLAMA_PLACEHOLDER_API_KEY: &str =
    super::transport_litellm::TEST_OLLAMA_PLACEHOLDER_API_KEY;

#[cfg(feature = "agent-provider-litellm")]
pub(crate) fn test_normalize_openai_compatible_base_url(api_base: &str) -> String {
    super::transport_litellm::test_normalize_openai_compatible_base_url(api_base)
}

#[cfg(feature = "agent-provider-litellm")]
pub(crate) fn test_normalize_litellm_embedding_target(
    model: &str,
    api_base: &str,
    api_key: Option<&str>,
) -> (String, String, Option<String>, bool) {
    super::transport_litellm::test_normalize_litellm_embedding_target(model, api_base, api_key)
}

pub(crate) async fn test_embed_http(
    client: &reqwest::Client,
    base_url: &str,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    transport_http::embed_http(client, base_url, texts, model).await
}
