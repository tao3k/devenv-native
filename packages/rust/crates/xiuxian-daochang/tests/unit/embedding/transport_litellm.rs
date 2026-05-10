//! Test coverage for xiuxian-daochang behavior.

use super::{
    OLLAMA_PLACEHOLDER_API_KEY, normalize_litellm_embedding_target,
    normalize_openai_compatible_base_url,
};

#[test]
fn normalize_openai_base_url_appends_v1_for_plain_host() {
    assert_eq!(
        normalize_openai_compatible_base_url("http://127.0.0.1:11434"),
        "http://127.0.0.1:11434/v1"
    );
}

#[test]
fn normalize_litellm_target_ollama_uses_openai_compat_with_placeholder_key() {
    let target = normalize_litellm_embedding_target(
        "ollama/qwen3-embedding:0.6b",
        "http://127.0.0.1:11434",
        None,
    );
    assert!(target.openai_compatible);
    assert_eq!(target.model, "openai/qwen3-embedding:0.6b");
    assert_eq!(target.api_base, "http://127.0.0.1:11434/v1");
    assert_eq!(target.api_key.as_deref(), Some(OLLAMA_PLACEHOLDER_API_KEY));
}

#[test]
fn normalize_litellm_target_non_ollama_is_passthrough() {
    let target = normalize_litellm_embedding_target(
        "minimax/text-embedding",
        "https://api.minimax.io/v1",
        Some("k"),
    );
    assert!(!target.openai_compatible);
    assert_eq!(target.model, "minimax/text-embedding");
    assert_eq!(target.api_base, "https://api.minimax.io/v1");
    assert_eq!(target.api_key.as_deref(), Some("k"));
}
