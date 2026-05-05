use crate::RuntimeSettings;

use super::runtime_guard::apply_channel_embedding_memory_guard_for_tests;

#[test]
fn channel_preserves_configured_http_embedding_backend() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.backend = Some("http".to_string());

    let resolved = apply_channel_embedding_memory_guard_for_tests(&settings, None, None, false);

    assert_eq!(resolved.memory.embedding_backend.as_deref(), Some("http"));
}

#[test]
fn channel_respects_explicit_memory_backend_env_override() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_backend = Some("http".to_string());

    let resolved =
        apply_channel_embedding_memory_guard_for_tests(&settings, Some("http"), None, false);

    assert_eq!(resolved.memory.embedding_backend.as_deref(), Some("http"));
}
