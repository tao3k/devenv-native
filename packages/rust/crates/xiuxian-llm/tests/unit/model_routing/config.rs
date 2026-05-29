use xiuxian_llm::model_routing::{
    WendaoModelRoutingMode, wendao_chat_route_config_with_model_routing_config,
    wendao_model_routing_config_from_toml_str, wendao_model_routing_mode_with_lookup,
    wendao_model_routing_system_default_config,
};

#[test]
fn model_routing_mode_defaults_to_local_deterministic_mode() -> Result<(), String> {
    assert_eq!(
        wendao_model_routing_mode_with_lookup(&|_| None)?,
        WendaoModelRoutingMode::Deterministic
    );
    assert_eq!(
        wendao_model_routing_mode_with_lookup(&|key| {
            (key == "WENDAO_MODEL_ROUTING_MODE").then(|| "vllm-sr".to_owned())
        })?,
        WendaoModelRoutingMode::VllmSr
    );
    assert!(wendao_model_routing_mode_with_lookup(&|_| Some("fallback".to_owned())).is_err());
    Ok(())
}

#[test]
fn model_routing_config_resolves_from_wendao_toml_before_env() -> Result<(), String> {
    let model_routing = wendao_model_routing_config_from_toml_str(
        r#"
        [model_routing]
        mode = "vllm-sr"
        vllm_sr_base_url = "http://127.0.0.1:8899"
        default_provider = "openrouter"

        [model_routing.chat]
        model = "deepseek/deepseek-v4-pro"
        backend_profile = "openai-compatible-chat-v1"

        [model_routing.audio_transcript]
        model = "qwen/qwen3-asr-flash-2026-02-10"
        backend_profile = "hosted-audio-transcript-v1"

        [model_routing.image_extract]
        provider = "openrouter-vision"
        model = "qwen/qwen3-vl-8b-instruct"
        backend_profile = "hosted-vlm-image-extract-v1"
        "#,
    )?;

    let chat =
        wendao_chat_route_config_with_model_routing_config(
            Some(&model_routing),
            &|key| match key {
                "WENDAO_MODEL_ROUTING_MODE" => Some("deterministic".to_owned()),
                "WENDAO_CHAT_ROUTE_MODEL" => Some("env-chat-model".to_owned()),
                _ => None,
            },
        )?;
    let audio =
        xiuxian_llm::model_routing::wendao_audio_transcript_route_config_with_model_routing_config(
            Some(&model_routing),
            &|_| None,
        )?;
    let image =
        xiuxian_llm::model_routing::wendao_image_extract_route_config_with_model_routing_config(
            Some(&model_routing),
            &|_| None,
        )?;

    assert_eq!(chat.model_routing_mode, WendaoModelRoutingMode::VllmSr);
    assert_eq!(chat.route_provider.as_deref(), Some("openrouter"));
    assert_eq!(chat.route_model, "deepseek/deepseek-v4-pro");
    assert_eq!(chat.vllm_sr_base_url, "http://127.0.0.1:8899");
    assert_eq!(audio.route_provider.as_deref(), Some("openrouter"));
    assert_eq!(audio.route_model, "qwen/qwen3-asr-flash-2026-02-10");
    assert_eq!(image.route_provider.as_deref(), Some("openrouter-vision"));
    assert_eq!(image.route_model, "qwen/qwen3-vl-8b-instruct");
    Ok(())
}

#[test]
fn model_routing_system_defaults_use_direct_deepseek_for_chat() -> Result<(), String> {
    let defaults = wendao_model_routing_system_default_config()?;
    assert_eq!(defaults.default_provider.as_deref(), Some("deepseek"));
    assert_eq!(defaults.chat.provider, None);
    assert_eq!(defaults.chat.model.as_deref(), Some("deepseek-chat"));
    assert_eq!(
        defaults.audio_transcript.provider.as_deref(),
        Some("openrouter")
    );
    assert_eq!(
        defaults.image_extract.provider.as_deref(),
        Some("openrouter")
    );

    let chat = wendao_chat_route_config_with_model_routing_config(None, &|_| None)?;
    assert_eq!(chat.route_provider.as_deref(), Some("deepseek"));
    assert_eq!(chat.route_model, "deepseek-chat");

    let env_chat = wendao_chat_route_config_with_model_routing_config(None, &|key| match key {
        "WENDAO_CHAT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        "WENDAO_CHAT_ROUTE_MODEL" => Some("env-chat-model".to_owned()),
        _ => None,
    })?;
    assert_eq!(env_chat.route_provider.as_deref(), Some("openrouter"));
    assert_eq!(env_chat.route_model, "env-chat-model");
    Ok(())
}
