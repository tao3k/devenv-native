//! LLM backend parsing tests.

use xiuxian_llm::llm::{LlmBackendKind, parse_llm_backend_kind};

#[test]
fn parse_llm_backend_kind_supports_http_aliases() {
    assert_eq!(
        parse_llm_backend_kind(Some("http")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("openai-compatible")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("openai_http")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("minimax")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("minimax-compatible")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
}

#[test]
fn parse_llm_backend_kind_supports_litellm_aliases() {
    assert_eq!(
        parse_llm_backend_kind(Some("litellm_rs")),
        Some(LlmBackendKind::LiteLlmRs)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("litellm-rs")),
        Some(LlmBackendKind::LiteLlmRs)
    );
}

#[test]
fn parse_llm_backend_kind_rejects_unknown_or_empty() {
    assert_eq!(parse_llm_backend_kind(Some("unsupported")), None);
    assert_eq!(parse_llm_backend_kind(Some("")), None);
    assert_eq!(parse_llm_backend_kind(None), None);
}
