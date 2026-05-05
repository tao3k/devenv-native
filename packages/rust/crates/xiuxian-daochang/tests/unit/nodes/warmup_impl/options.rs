use xiuxian_daochang::RuntimeSettings;
use xiuxian_daochang::test_support::{WarmupEnvOverrides, resolve_warmup_options};

#[test]
fn resolve_warmup_options_prefers_memory_backend_and_model() {
    let mut runtime = RuntimeSettings::default();
    runtime.memory.embedding_backend = Some("http".to_string());
    runtime.memory.embedding_model = Some("Qwen/Qwen3-Embedding-0.6B".to_string());
    runtime.memory.embedding_base_url = Some("http://127.0.0.1:19092".to_string());
    runtime.embedding.timeout_secs = Some(21);
    runtime.embedding.batch_max_size = Some(256);
    runtime.embedding.batch_max_concurrency = Some(8);

    let options = resolve_warmup_options(&runtime, &WarmupEnvOverrides::default(), None);
    assert_eq!(options.backend_hint.as_deref(), Some("http"));
    assert_eq!(options.model.as_deref(), Some("Qwen/Qwen3-Embedding-0.6B"));
    assert_eq!(options.base_url, "http://127.0.0.1:19092");
    assert_eq!(options.timeout_secs, 21);
    assert_eq!(options.batch_max_size, Some(256));
    assert_eq!(options.batch_max_concurrency, Some(8));
}

#[test]
fn resolve_warmup_options_prefers_env_overrides() {
    let mut runtime = RuntimeSettings::default();
    runtime.memory.embedding_backend = Some("http".to_string());
    runtime.embedding.timeout_secs = Some(12);
    runtime.embedding.batch_max_size = Some(64);
    runtime.embedding.batch_max_concurrency = Some(2);

    let env = WarmupEnvOverrides {
        memory_embedding_backend: Some("openai_http".to_string()),
        memory_embedding_model: Some("env-model".to_string()),
        memory_embedding_base_url: Some("http://127.0.0.1:18092".to_string()),
        embed_timeout_secs: Some(33),
        memory_embed_batch_max_size: Some(512),
        memory_embed_batch_max_concurrency: Some(16),
        ..WarmupEnvOverrides::default()
    };

    let options = resolve_warmup_options(&runtime, &env, Some("cli-model"));
    assert_eq!(options.backend_hint.as_deref(), Some("openai_http"));
    assert_eq!(options.model.as_deref(), Some("cli-model"));
    assert_eq!(options.base_url, "http://127.0.0.1:18092");
    assert_eq!(options.timeout_secs, 33);
    assert_eq!(options.batch_max_size, Some(512));
    assert_eq!(options.batch_max_concurrency, Some(16));
}

#[test]
fn resolve_warmup_options_uses_defaults_when_missing() {
    let runtime = RuntimeSettings::default();
    let options = resolve_warmup_options(&runtime, &WarmupEnvOverrides::default(), None);
    assert!(options.backend_hint.is_none());
    assert!(options.model.is_none());
    assert_eq!(options.base_url, "http://127.0.0.1:3002");
    assert_eq!(options.timeout_secs, 15);
}
