//! Embedding helpers exposed for integration tests.

/// Embedding backend mode alias used by embedding tests.
pub type EmbeddingBackendMode = xiuxian_llm::embedding::EmbeddingBackendKind;

#[must_use]
/// Parses an embedding backend mode for tests.
pub fn parse_embedding_client_backend_mode(raw: Option<&str>) -> EmbeddingBackendMode {
    crate::embedding::test_parse_backend_mode(raw)
}

#[cfg(feature = "agent-provider-litellm")]
/// Placeholder API key used for Ollama-compatible embedding tests.
pub const OLLAMA_PLACEHOLDER_API_KEY: &str = crate::embedding::TEST_OLLAMA_PLACEHOLDER_API_KEY;

#[cfg(feature = "agent-provider-litellm")]
#[must_use]
/// Normalizes an OpenAI-compatible embedding base URL.
pub fn normalize_openai_compatible_base_url(api_base: &str) -> String {
    crate::embedding::test_normalize_openai_compatible_base_url(api_base)
}

#[cfg(feature = "agent-provider-litellm")]
#[must_use]
/// Normalizes a LiteLLM embedding target tuple.
pub fn normalize_litellm_embedding_target(
    model: &str,
    api_base: &str,
    api_key: Option<&str>,
) -> (String, String, Option<String>, bool) {
    crate::embedding::test_normalize_litellm_embedding_target(model, api_base, api_key)
}

/// Executes one HTTP embedding request for tests.
pub async fn embed_http(
    client: &reqwest::Client,
    base_url: &str,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    crate::embedding::test_embed_http(client, base_url, texts, model).await
}
