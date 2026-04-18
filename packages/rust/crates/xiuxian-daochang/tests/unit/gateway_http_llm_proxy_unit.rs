//! Top-level integration tests for `gateway::http::llm_proxy`.

use xiuxian_daochang::test_support::{
    read_api_key, resolve_request_model, resolve_target_api_key_env, resolve_target_base_url,
};

/// Verifies that an explicit override is trimmed before use.
#[test]
fn resolve_target_base_url_prefers_trimmed_override() {
    let resolved =
        resolve_target_base_url(Some("  http://127.0.0.1:11434/v1  "), "http://fallback");
    assert_eq!(resolved, "http://127.0.0.1:11434/v1");
}

/// Verifies that a blank override falls back to the runtime default.
#[test]
fn resolve_target_base_url_falls_back_when_override_blank() {
    let resolved = resolve_target_base_url(Some("   "), "http://fallback");
    assert_eq!(resolved, "http://fallback");
}

/// Verifies that a blank API-key env override falls back to the default env name.
#[test]
fn resolve_target_api_key_env_falls_back_when_override_blank() {
    let resolved = resolve_target_api_key_env(Some("   "), "OPENAI_API_KEY");
    assert_eq!(resolved, "OPENAI_API_KEY");
}

/// Verifies that missing environment variables resolve to an empty key.
#[test]
fn read_api_key_returns_empty_for_missing_env() {
    let resolved = read_api_key("XIUXIAN_DAOCHANG_TEST_LLM_PROXY_UNLIKELY_TO_EXIST_KEY");
    assert!(resolved.is_empty());
}

/// Verifies that literal key values are passed through unchanged.
#[test]
fn read_api_key_returns_literal_value() {
    let resolved = read_api_key("sk-test-literal-key");
    assert_eq!(resolved, "sk-test-literal-key");
}

/// Verifies that request-level model selection has highest priority.
#[test]
fn resolve_request_model_prefers_request_value() {
    let model = resolve_request_model(
        Some("minimax/MiniMax-M2.5"),
        Some("settings-model"),
        Some("xiuxian-model"),
    );
    assert_eq!(model.as_deref(), Some("minimax/MiniMax-M2.5"));
}

/// Verifies that settings-level default applies when request model is absent.
#[test]
fn resolve_request_model_uses_settings_default_when_request_missing() {
    let model = resolve_request_model(None, Some("settings-model"), Some("xiuxian-model"));
    assert_eq!(model.as_deref(), Some("settings-model"));
}

/// Verifies fallback to xiuxian model when request and settings values are blank.
#[test]
fn resolve_request_model_uses_xiuxian_default_when_request_and_settings_missing() {
    let model = resolve_request_model(Some("  "), Some(""), Some("xiuxian-model"));
    assert_eq!(model.as_deref(), Some("xiuxian-model"));
}
