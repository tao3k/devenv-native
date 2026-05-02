//! Verifies the default `xiuxian-llm` feature profile stays on the LiteLLM-only path.

#[test]
fn default_feature_profile_keeps_provider_litellm_enabled() {
    let provider_litellm_enabled = std::hint::black_box(cfg!(feature = "provider-litellm"));
    assert!(provider_litellm_enabled);
    let _ = std::mem::size_of::<xiuxian_llm::llm::OpenAICompatibleClient>();
    let backend = xiuxian_llm::embedding::parse_embedding_backend_kind(Some("litellm"));
    assert_eq!(
        backend,
        Some(xiuxian_llm::embedding::EmbeddingBackendKind::LiteLlmRs)
    );
}
