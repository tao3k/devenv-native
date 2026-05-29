//! Tests for OpenAI-compatible runtime profile resolution.

use std::collections::HashMap;
use xiuxian_llm::llm::{
    LlmProviderProfileInput, LlmRuntimeDefaults, LlmRuntimeProfileEnv, LlmRuntimeProfileInput,
    OpenAIWireApi, llm_runtime_profile_input_from_toml_config,
    llm_runtime_profile_system_default_config, llm_runtime_profile_toml_config_from_str,
    resolve_openai_runtime_profile, runtime_profile_env_with_model_decision,
};
use xiuxian_llm::model_routing::WendaoModelDecision;

#[test]
fn runtime_profile_resolves_default_provider_and_responses_wire() {
    let mut providers = HashMap::new();
    providers.insert(
        "crs".to_string(),
        LlmProviderProfileInput {
            model: Some("gpt-5-codex".to_string()),
            base_url: Some("https://openai-compatible.example.com/v1".to_string()),
            api_key: Some("CRS_OAI_KEY".to_string()),
            api_key_env: None,
            wire_api: Some("responses".to_string()),
        },
    );
    let profile = LlmRuntimeProfileInput {
        model: None,
        default_model: None,
        base_url: None,
        api_key_env: None,
        api_key: None,
        wire_api: None,
        default_provider: Some("crs".to_string()),
        providers,
    };
    let env = LlmRuntimeProfileEnv {
        provider_override: None,
        model_override: None,
        base_url_override: None,
        api_key_override: None,
        wire_api_override: None,
        env_vars: vec![
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("CRS_OAI_KEY".to_string(), "crs-secret".to_string()),
        ],
    };
    let defaults = LlmRuntimeDefaults {
        provider: "openai".to_string(),
        model: "fallback-model".to_string(),
        base_url: "http://localhost:3002/v1".to_string(),
        api_key_env: "OPENAI_API_KEY".to_string(),
        wire_api: OpenAIWireApi::ChatCompletions,
    };

    let resolved = resolve_openai_runtime_profile(&profile, &env, &defaults)
        .unwrap_or_else(|err| panic!("runtime profile resolution should succeed: {err}"));

    assert_eq!(resolved.provider_name, "crs");
    assert_eq!(resolved.model, "gpt-5-codex");
    assert_eq!(
        resolved.base_url,
        "https://openai-compatible.example.com/v1"
    );
    assert_eq!(resolved.api_key_env, "CRS_OAI_KEY");
    assert_eq!(resolved.api_key, "crs-secret");
    assert_eq!(resolved.wire_api, OpenAIWireApi::Responses);
}

#[test]
fn runtime_profile_system_defaults_use_direct_deepseek_provider() {
    let config = llm_runtime_profile_system_default_config()
        .unwrap_or_else(|_| panic!("source LLM defaults should parse"));
    assert_eq!(config.backend.as_deref(), Some("litellm"));
    assert_eq!(config.default_provider.as_deref(), Some("deepseek"));
    assert_eq!(config.default_model.as_deref(), Some("deepseek-chat"));
    assert!(config.providers.contains_key("deepseek"));
    assert!(config.providers.contains_key("openrouter"));
    assert!(config.providers.contains_key("local_openai"));

    let profile = llm_runtime_profile_input_from_toml_config(config);
    let env = LlmRuntimeProfileEnv {
        env_vars: vec![(
            "DEEPSEEK_API_KEY".to_string(),
            "deepseek-secret".to_string(),
        )],
        ..LlmRuntimeProfileEnv::default()
    };
    let resolved = resolve_openai_runtime_profile(&profile, &env, &LlmRuntimeDefaults::default())
        .unwrap_or_else(|_| panic!("DeepSeek provider should resolve from source defaults"));

    assert_eq!(resolved.provider_name, "deepseek");
    assert_eq!(resolved.model, "deepseek-chat");
    assert_eq!(resolved.base_url, "https://api.deepseek.com/v1");
    assert_eq!(resolved.api_key_env, "DEEPSEEK_API_KEY");
    assert_eq!(resolved.wire_api, OpenAIWireApi::ChatCompletions);
}

#[test]
fn runtime_profile_project_toml_can_select_openrouter_provider() {
    let toml = r#"
        [llm]
        backend = "litellm"
        default_provider = "openrouter"
        default_model = "deepseek/deepseek-v4-pro"
        wire_api = "chat_completions"

        [llm.providers.openrouter]
        model = "deepseek/deepseek-v4-pro"
        base_url = "https://openrouter.ai/api/v1"
        api_key_env = "OPENROUTER_API_KEY"
        wire_api = "chat_completions"
    "#;

    let config = llm_runtime_profile_toml_config_from_str(toml)
        .unwrap_or_else(|_| panic!("wendao.toml LLM config should parse"));
    assert_eq!(config.backend.as_deref(), Some("litellm"));
    assert_eq!(config.default_provider.as_deref(), Some("openrouter"));
    assert!(config.providers.contains_key("deepseek"));

    let profile = llm_runtime_profile_input_from_toml_config(config);
    let env = LlmRuntimeProfileEnv {
        env_vars: vec![(
            "OPENROUTER_API_KEY".to_string(),
            "openrouter-secret".to_string(),
        )],
        ..LlmRuntimeProfileEnv::default()
    };
    let resolved = resolve_openai_runtime_profile(&profile, &env, &LlmRuntimeDefaults::default())
        .unwrap_or_else(|_| panic!("OpenRouter provider should resolve from wendao.toml"));

    assert_eq!(resolved.provider_name, "openrouter");
    assert_eq!(resolved.model, "deepseek/deepseek-v4-pro");
    assert_eq!(resolved.base_url, "https://openrouter.ai/api/v1");
    assert_eq!(resolved.api_key_env, "OPENROUTER_API_KEY");
    assert_eq!(resolved.wire_api, OpenAIWireApi::ChatCompletions);
}

#[test]
fn runtime_profile_missing_api_key_env_fails() {
    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        LlmProviderProfileInput {
            model: Some("gpt-5-codex".to_string()),
            base_url: Some("https://openai-compatible.example.com/v1".to_string()),
            api_key: Some("OPENAI_API_KEY".to_string()),
            api_key_env: None,
            wire_api: Some("responses".to_string()),
        },
    );
    let profile = LlmRuntimeProfileInput {
        model: None,
        default_model: None,
        base_url: None,
        api_key_env: None,
        api_key: None,
        wire_api: None,
        default_provider: Some("openai".to_string()),
        providers,
    };
    let env = LlmRuntimeProfileEnv {
        provider_override: None,
        model_override: None,
        base_url_override: None,
        api_key_override: None,
        wire_api_override: None,
        env_vars: vec![("OPENAI_API_KEY".to_string(), String::new())],
    };
    let defaults = LlmRuntimeDefaults::default();

    let err = match resolve_openai_runtime_profile(&profile, &env, &defaults) {
        Ok(profile) => panic!("expected missing API key error, got: {profile:?}"),
        Err(err) => err,
    };
    let text = err.to_string();
    assert!(
        text.contains("missing LLM API key"),
        "unexpected error: {text}"
    );
}

#[test]
fn runtime_profile_wire_override_takes_precedence() {
    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        LlmProviderProfileInput {
            model: Some("gpt-5-codex".to_string()),
            base_url: Some("https://openai-compatible.example.com/v1".to_string()),
            api_key: Some("OPENAI_API_KEY".to_string()),
            api_key_env: None,
            wire_api: Some("responses".to_string()),
        },
    );
    let profile = LlmRuntimeProfileInput {
        model: None,
        default_model: None,
        base_url: None,
        api_key_env: None,
        api_key: None,
        wire_api: None,
        default_provider: Some("openai".to_string()),
        providers,
    };
    let env = LlmRuntimeProfileEnv {
        provider_override: None,
        model_override: None,
        base_url_override: None,
        api_key_override: None,
        wire_api_override: Some("chat_completions".to_string()),
        env_vars: vec![("OPENAI_API_KEY".to_string(), "test-openai-key".to_string())],
    };

    let resolved = resolve_openai_runtime_profile(&profile, &env, &LlmRuntimeDefaults::default())
        .unwrap_or_else(|err| panic!("runtime profile resolution should succeed: {err}"));
    assert_eq!(resolved.wire_api, OpenAIWireApi::ChatCompletions);
}

#[test]
fn runtime_profile_prefers_provider_specific_model_and_base_url_over_flat_defaults() {
    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        LlmProviderProfileInput {
            model: Some("mimo-v2-pro".to_string()),
            base_url: Some("https://token-plan-sgp.xiaomimimo.com/v1".to_string()),
            api_key: Some("MIMO_API_KEY".to_string()),
            api_key_env: None,
            wire_api: None,
        },
    );
    let profile = LlmRuntimeProfileInput {
        model: Some("system-flat-model".to_string()),
        default_model: Some("system-default-model".to_string()),
        base_url: Some("http://localhost:3002/v1".to_string()),
        api_key_env: Some("SYSTEM_API_KEY".to_string()),
        api_key: None,
        wire_api: None,
        default_provider: Some("openai".to_string()),
        providers,
    };
    let env = LlmRuntimeProfileEnv {
        provider_override: None,
        model_override: None,
        base_url_override: None,
        api_key_override: None,
        wire_api_override: None,
        env_vars: vec![
            (
                "OPENAI_API_KEY".to_string(),
                "generic-openai-secret".to_string(),
            ),
            ("MIMO_API_KEY".to_string(), "mimo-secret".to_string()),
            ("SYSTEM_API_KEY".to_string(), "system-secret".to_string()),
        ],
    };
    let defaults = LlmRuntimeDefaults::default();

    let resolved = resolve_openai_runtime_profile(&profile, &env, &defaults)
        .unwrap_or_else(|err| panic!("runtime profile resolution should succeed: {err}"));

    assert_eq!(resolved.model, "mimo-v2-pro");
    assert_eq!(
        resolved.base_url,
        "https://token-plan-sgp.xiaomimimo.com/v1"
    );
    assert_eq!(resolved.api_key_env, "MIMO_API_KEY");
    assert_eq!(resolved.api_key, "mimo-secret");
}

#[test]
fn runtime_profile_model_route_decision_overrides_provider_and_model_only() {
    let mut providers = HashMap::new();
    providers.insert(
        "openrouter".to_string(),
        LlmProviderProfileInput {
            model: Some("configured-chat-model".to_string()),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_key: Some("OPENROUTER_API_KEY".to_string()),
            api_key_env: None,
            wire_api: Some("chat_completions".to_string()),
        },
    );
    providers.insert(
        "openai".to_string(),
        LlmProviderProfileInput {
            model: Some("gpt-4o-mini".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_key: Some("OPENAI_API_KEY".to_string()),
            api_key_env: None,
            wire_api: None,
        },
    );
    let profile = LlmRuntimeProfileInput {
        model: None,
        default_model: None,
        base_url: None,
        api_key_env: None,
        api_key: None,
        wire_api: None,
        default_provider: Some("openai".to_string()),
        providers,
    };
    let env = LlmRuntimeProfileEnv {
        provider_override: None,
        model_override: None,
        base_url_override: None,
        api_key_override: None,
        wire_api_override: Some("responses".to_string()),
        env_vars: vec![
            (
                "OPENROUTER_API_KEY".to_string(),
                "openrouter-secret".to_string(),
            ),
            ("OPENAI_API_KEY".to_string(), "openai-secret".to_string()),
        ],
    };
    let decision = WendaoModelDecision {
        route_id: "route-chat-1".to_string(),
        selected_provider: "openrouter".to_string(),
        selected_model: "deepseek/deepseek-v4-pro".to_string(),
        selected_backend_profile: "openai-compatible-chat-v1".to_string(),
        reasoning_policy: Some("high".to_string()),
        route_trace: Some("chat model card".to_string()),
    };

    let routed_env = runtime_profile_env_with_model_decision(&env, &decision);
    let resolved =
        resolve_openai_runtime_profile(&profile, &routed_env, &LlmRuntimeDefaults::default())
            .unwrap_or_else(|err| panic!("runtime profile resolution should succeed: {err}"));

    assert_eq!(resolved.provider_name, "openrouter");
    assert_eq!(resolved.model, "deepseek/deepseek-v4-pro");
    assert_eq!(resolved.base_url, "https://openrouter.ai/api/v1");
    assert_eq!(resolved.api_key_env, "OPENROUTER_API_KEY");
    assert_eq!(resolved.api_key, "openrouter-secret");
    assert_eq!(resolved.wire_api, OpenAIWireApi::Responses);
}
