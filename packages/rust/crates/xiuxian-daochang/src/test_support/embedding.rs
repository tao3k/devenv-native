//! Embedding helpers exposed for integration tests.

/// Embedding backend mode alias used by embedding tests.
pub type EmbeddingBackendMode = xiuxian_llm::embedding::EmbeddingBackendKind;

/// Parses an embedding backend mode for tests.
#[must_use]
pub fn parse_embedding_client_backend_mode(raw: Option<&str>) -> EmbeddingBackendMode {
    crate::embedding::test_parse_backend_mode(raw)
}

#[cfg(feature = "agent-provider-litellm")]
/// Placeholder API key used for Ollama-compatible embedding tests.
pub const OLLAMA_PLACEHOLDER_API_KEY: &str = crate::embedding::TEST_OLLAMA_PLACEHOLDER_API_KEY;

#[cfg(feature = "agent-provider-litellm")]
/// Normalizes an OpenAI-compatible embedding base URL.
#[must_use]
pub fn normalize_openai_compatible_base_url(api_base: &str) -> String {
    crate::embedding::test_normalize_openai_compatible_base_url(api_base)
}

#[cfg(feature = "agent-provider-litellm")]
/// Normalized `LiteLLM` embedding target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedLitellmEmbeddingTarget {
    /// Effective model id.
    pub model: String,
    /// Effective API base URL.
    pub api_base: String,
    /// Effective API key when one is required.
    pub api_key: Option<String>,
    /// Whether the target uses OpenAI-compatible transport.
    pub openai_compatible: bool,
}

#[cfg(feature = "agent-provider-litellm")]
/// Normalizes a `LiteLLM` embedding target.
#[must_use]
pub fn normalize_litellm_embedding_target(
    model: &str,
    api_base: &str,
    api_key: Option<&str>,
) -> NormalizedLitellmEmbeddingTarget {
    let (model, api_base, api_key, openai_compatible) =
        crate::embedding::test_normalize_litellm_embedding_target(model, api_base, api_key);
    NormalizedLitellmEmbeddingTarget {
        model,
        api_base,
        api_key,
        openai_compatible,
    }
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
