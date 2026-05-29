//! Verifies the default `xiuxian-llm` feature profile keeps routing and `LiteLLM` enabled.

#[test]
fn default_feature_profile_enables_model_routing_and_litellm_provider() {
    let model_routing_enabled = std::hint::black_box(cfg!(feature = "model-routing"));
    assert!(model_routing_enabled);
    let provider_litellm_enabled = std::hint::black_box(cfg!(feature = "provider-litellm"));
    assert!(provider_litellm_enabled);
    let _ = std::mem::size_of::<xiuxian_llm::llm::OpenAICompatibleClient>();
    let backend = xiuxian_llm::embedding::parse_embedding_backend_kind(Some("litellm"));
    assert_eq!(
        backend,
        Some(xiuxian_llm::embedding::EmbeddingBackendKind::LiteLlmRs)
    );
}
