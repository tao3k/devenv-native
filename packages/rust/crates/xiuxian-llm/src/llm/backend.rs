//! LLM backend mode parsing and normalized backend kinds.

/// Normalized LLM backend kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmBackendKind {
    /// OpenAI-compatible HTTP endpoint.
    OpenAiCompatibleHttp,
    /// `litellm-rs` provider path.
    LiteLlmRs,
}

impl LlmBackendKind {
    /// Stable config value for this backend.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAiCompatibleHttp => "http",
            Self::LiteLlmRs => "litellm_rs",
        }
    }
}

/// Parse a backend hint into normalized LLM backend kind.
///
/// Returns `None` when the input is empty or not recognized.
#[must_use]
pub fn parse_llm_backend_kind(raw: Option<&str>) -> Option<LlmBackendKind> {
    let normalized = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)?;

    if normalized == "http"
        || normalized == "openai_compatible"
        || normalized == "openai-compatible"
        || normalized == "openai_http"
        || normalized == "openai-http"
        || normalized == "openai"
        || normalized == "minimax"
        || normalized == "minimax_http"
        || normalized == "minimax-http"
        || normalized == "minimax_compatible"
        || normalized == "minimax-compatible"
    {
        return Some(LlmBackendKind::OpenAiCompatibleHttp);
    }

    if normalized == "litellm_rs" || normalized == "litellm-rs" {
        return Some(LlmBackendKind::LiteLlmRs);
    }

    None
}
