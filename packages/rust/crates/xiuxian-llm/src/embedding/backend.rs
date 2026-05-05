//! Embedding backend mode parsing and normalized backend kinds.

/// Normalized embedding backend kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingBackendKind {
    /// Legacy `/embed/batch` HTTP endpoint.
    Http,
    /// OpenAI-compatible `/v1/embeddings` endpoint.
    OpenAiHttp,
    /// `litellm-rs` in-process provider path.
    LiteLlmRs,
}

impl EmbeddingBackendKind {
    /// Stable config value for this backend.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::OpenAiHttp => "openai_http",
            Self::LiteLlmRs => "litellm_rs",
        }
    }
}

/// Parse a backend hint into a normalized embedding backend kind.
///
/// Returns `None` when the input is empty or not recognized.
#[must_use]
pub fn parse_embedding_backend_kind(raw: Option<&str>) -> Option<EmbeddingBackendKind> {
    let normalized = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)?;

    if normalized == "http" || normalized == "client" {
        return Some(EmbeddingBackendKind::Http);
    }

    if normalized == "openai_http"
        || normalized == "openai-http"
        || normalized == "openai_compat"
        || normalized == "openai-compatible"
        || normalized == "openai"
    {
        return Some(EmbeddingBackendKind::OpenAiHttp);
    }

    if normalized == "litellm_rs"
        || normalized == "litellm-rs"
        || normalized == "litellm"
        || normalized == "provider"
    {
        return Some(EmbeddingBackendKind::LiteLlmRs);
    }

    None
}
