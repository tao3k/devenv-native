use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use xiuxian_llm::model_routing::{
    VLLM_SR_SELECTED_CONFIDENCE_HEADER, VLLM_SR_SELECTED_DECISION_HEADER,
    VLLM_SR_SELECTED_MODALITY_HEADER, VLLM_SR_SELECTED_MODEL_HEADER,
    VLLM_SR_SELECTED_REASONING_HEADER, VllmSrRouteDecisionClient, WENDAO_ROUTE_ID_HEADER,
    WENDAO_ROUTE_MODALITY_HEADER, WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
    WENDAO_ROUTE_SELECTED_MODEL_HEADER, WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
    WendaoChatRouteInput, WendaoModelDecision, WendaoModelRoutingMode, WendaoRouteIntent,
    wendao_attachment_model_route_decision, wendao_audio_transcript_route_config_with_lookup,
    wendao_audio_transcript_route_config_with_model_routing_config,
    wendao_chat_route_config_with_lookup, wendao_chat_route_config_with_model_routing_config,
    wendao_chat_route_intent, wendao_image_extract_route_config_with_lookup,
    wendao_image_extract_route_config_with_model_routing_config, wendao_model_route_metadata,
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
fn model_decision_parses_vllm_sr_response_shape() -> Result<(), String> {
    let decision = WendaoModelDecision::from_vllm_sr_response_json(
        r#"{
          "decision": {
            "routeId": "route-audio-1",
            "selectedProvider": "openrouter",
            "selectedModel": "qwen/qwen3-asr-flash-2026-02-10",
            "selectedBackendProfile": "hosted-audio-transcript-v1",
            "reasoningPolicy": "none",
            "routeTrace": "matched audio card"
          }
        }"#,
    )?;

    assert_eq!(decision.route_id, "route-audio-1");
    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "qwen/qwen3-asr-flash-2026-02-10");
    assert_eq!(
        decision.selected_backend_profile,
        "hosted-audio-transcript-v1"
    );
    assert_eq!(decision.reasoning_policy.as_deref(), Some("none"));
    Ok(())
}

#[test]
fn model_route_metadata_preserves_gateway_intent_and_decision() {
    let intent = WendaoRouteIntent {
        task_kind: "attachment-extract".into(),
        modality: "audio".to_owned(),
        source_kind: "attachment".into(),
        precision_tier: "high".to_owned(),
        privacy_tier: "private".to_owned(),
        latency_budget_ms: 120_000,
        evidence_profile: "transcript".to_owned(),
        artifact_refs: vec!["artifact://audio/001".to_owned()],
    };
    let decision = WendaoModelDecision {
        route_id: "route-audio-1".to_owned(),
        selected_provider: "openrouter".to_owned(),
        selected_model: "qwen/qwen3-asr-flash-2026-02-10".to_owned(),
        selected_backend_profile: "hosted-audio-transcript-v1".to_owned(),
        reasoning_policy: Some("none".to_owned()),
        route_trace: Some("matched audio card".to_owned()),
    };

    let metadata = wendao_model_route_metadata(&intent, &decision);

    assert!(metadata.contains(&(WENDAO_ROUTE_ID_HEADER, "route-audio-1".to_owned())));
    assert!(metadata.contains(&(WENDAO_ROUTE_MODALITY_HEADER, "audio".to_owned())));
    assert!(metadata.contains(&(
        WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
        "openrouter".to_owned()
    )));
    assert!(metadata.contains(&(
        WENDAO_ROUTE_SELECTED_MODEL_HEADER,
        "qwen/qwen3-asr-flash-2026-02-10".to_owned(),
    )));
    assert!(metadata.contains(&(
        WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
        "hosted-audio-transcript-v1".to_owned(),
    )));
}

#[test]
fn chat_route_config_and_intent_are_gateway_owned() -> Result<(), String> {
    let config = wendao_chat_route_config_with_lookup(&|key| match key {
        "WENDAO_MODEL_ROUTING_MODE" => Some("vllm-sr".to_owned()),
        "WENDAO_CHAT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        "WENDAO_CHAT_ROUTE_MODEL" => Some("deepseek/deepseek-v4-pro".to_owned()),
        "WENDAO_CHAT_ROUTE_BACKEND_PROFILE" => Some("openai-compatible-chat-v1".to_owned()),
        "WENDAO_VLLM_SR_BASE_URL" => Some("http://127.0.0.1:8888/".to_owned()),
        _ => None,
    })?;
    let input = WendaoChatRouteInput {
        artifact_refs: vec!["artifact://conversation/session-1".to_owned()],
        ..WendaoChatRouteInput::default()
    };

    let intent = wendao_chat_route_intent(&input);

    assert_eq!(config.route_provider.as_deref(), Some("openrouter"));
    assert_eq!(config.route_model, "deepseek/deepseek-v4-pro");
    assert_eq!(config.backend_profile, "openai-compatible-chat-v1");
    assert_eq!(config.vllm_sr_base_url, "http://127.0.0.1:8888");
    assert_eq!(intent.task_kind.as_str(), "chat");
    assert_eq!(intent.modality, "text");
    assert_eq!(intent.source_kind.as_str(), "conversation");
    assert_eq!(intent.evidence_profile, "local-knowledge-chat");
    assert_eq!(
        intent.artifact_refs,
        vec!["artifact://conversation/session-1".to_owned()]
    );
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
    let audio = wendao_audio_transcript_route_config_with_model_routing_config(
        Some(&model_routing),
        &|_| None,
    )?;
    let image =
        wendao_image_extract_route_config_with_model_routing_config(Some(&model_routing), &|_| {
            None
        })?;

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

#[tokio::test]
async fn chat_route_deterministic_mode_returns_gateway_decision() -> Result<(), String> {
    let config = wendao_chat_route_config_with_lookup(&|key| match key {
        "WENDAO_CHAT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        "WENDAO_CHAT_ROUTE_MODEL" => Some("deepseek/deepseek-v4-pro".to_owned()),
        _ => None,
    })?;

    let (intent, decision) = xiuxian_llm::model_routing::wendao_chat_model_route_decision(
        &config,
        &WendaoChatRouteInput::default(),
    )
    .await?;

    assert_eq!(
        config.model_routing_mode,
        WendaoModelRoutingMode::Deterministic
    );
    assert_eq!(intent.task_kind.as_str(), "chat");
    assert_eq!(
        decision.route_id,
        "deterministic:chat:openrouter:deepseek-deepseek-v4-pro"
    );
    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "deepseek/deepseek-v4-pro");
    assert_eq!(
        decision.selected_backend_profile,
        "openai-compatible-chat-v1"
    );
    assert!(
        decision
            .route_trace
            .as_deref()
            .unwrap_or_default()
            .contains("deterministic")
    );
    Ok(())
}

#[tokio::test]
async fn audio_attachment_route_deterministic_mode_returns_gateway_decision() -> Result<(), String>
{
    let config = wendao_audio_transcript_route_config_with_lookup(&|key| match key {
        "WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        "WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL" => Some("qwen/qwen3-asr-flash-2026-02-10".to_owned()),
        _ => None,
    })?;
    let input = xiuxian_llm::model_routing::WendaoAttachmentRouteInput {
        task_kind: "attachment-extract".to_owned(),
        modality: "audio".to_owned(),
        source_kind: "attachment".to_owned(),
        precision_tier: "high".to_owned(),
        privacy_tier: "private".to_owned(),
        latency_budget_ms: 120_000,
        evidence_profile: "audio-transcript".to_owned(),
        artifact_refs: vec!["source-sha256:abc".to_owned()],
    };

    let (intent, decision) = wendao_attachment_model_route_decision(&config, &input).await?;

    assert_eq!(
        config.model_routing_mode,
        WendaoModelRoutingMode::Deterministic
    );
    assert_eq!(intent.modality, "audio");
    assert_eq!(
        decision.route_id,
        "deterministic:attachment-extract:audio:openrouter:qwen-qwen3-asr-flash-2026-02-10"
    );
    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "qwen/qwen3-asr-flash-2026-02-10");
    assert_eq!(
        decision.selected_backend_profile,
        "hosted-audio-transcript-v1"
    );
    Ok(())
}

#[tokio::test]
async fn image_attachment_route_deterministic_mode_returns_gateway_decision() -> Result<(), String>
{
    let config = wendao_image_extract_route_config_with_lookup(&|key| match key {
        "WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        "WENDAO_IMAGE_EXTRACT_ROUTE_MODEL" => Some("qwen/qwen3-vl-8b-instruct".to_owned()),
        _ => None,
    })?;
    let input = xiuxian_llm::model_routing::WendaoAttachmentRouteInput {
        task_kind: "attachment-extract".to_owned(),
        modality: "image".to_owned(),
        source_kind: "attachment".to_owned(),
        precision_tier: "high".to_owned(),
        privacy_tier: "private".to_owned(),
        latency_budget_ms: 60_000,
        evidence_profile: "image-document-markdown".to_owned(),
        artifact_refs: vec!["source-suffix:png".to_owned()],
    };

    let (intent, decision) = wendao_attachment_model_route_decision(&config, &input).await?;

    assert_eq!(intent.modality, "image");
    assert_eq!(
        decision.route_id,
        "deterministic:attachment-extract:image:openrouter:qwen-qwen3-vl-8b-instruct"
    );
    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "qwen/qwen3-vl-8b-instruct");
    assert_eq!(
        decision.selected_backend_profile,
        "hosted-vlm-image-extract-v1"
    );
    Ok(())
}

#[tokio::test]
async fn chat_route_config_requires_provider_hint_in_vllm_sr_mode() -> Result<(), String> {
    let config = wendao_chat_route_config_with_lookup(&|key| {
        (key == "WENDAO_MODEL_ROUTING_MODE").then(|| "vllm-sr".to_owned())
    })?;

    let error = xiuxian_llm::model_routing::wendao_chat_model_route_decision(
        &config,
        &WendaoChatRouteInput::default(),
    )
    .await
    .expect_err("vLLM-SR chat route should require provider hint");

    assert!(error.contains("WENDAO_CHAT_ROUTE_PROVIDER"));
    Ok(())
}

#[tokio::test]
async fn vllm_sr_route_probe_parses_official_selection_headers() -> Result<(), String> {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| error.to_string())?;
    let addr = listener.local_addr().map_err(|error| error.to_string())?;
    let app = Router::new()
        .route("/v1/chat/completions", post(vllm_sr_chat_probe))
        .with_state(Arc::clone(&requests));
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let client = VllmSrRouteDecisionClient::new(format!("http://{addr}"));
    let intent = WendaoRouteIntent {
        task_kind: "attachment-extract".into(),
        modality: "audio".to_owned(),
        source_kind: "attachment".into(),
        precision_tier: "high".to_owned(),
        privacy_tier: "private".to_owned(),
        latency_budget_ms: 120_000,
        evidence_profile: "transcript".to_owned(),
        artifact_refs: vec!["artifact://audio/001".to_owned()],
    };

    let decision = client
        .decide(&intent, "openrouter", "hosted-audio-transcript-v1")
        .await?;

    assert_eq!(decision.route_id, "vllm-sr:audio-decision:qwen-qwen3-asr");
    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "qwen/qwen3-asr");
    assert_eq!(
        decision.selected_backend_profile,
        "hosted-audio-transcript-v1"
    );
    assert_eq!(decision.reasoning_policy.as_deref(), Some("off"));
    assert!(
        decision
            .route_trace
            .as_deref()
            .unwrap_or_default()
            .contains("audio-decision")
    );

    let captured = requests
        .lock()
        .map_err(|_| "request capture lock poisoned".to_owned())?;
    let payload = captured
        .first()
        .ok_or_else(|| "expected vLLM-SR route probe request".to_owned())?;
    assert_eq!(payload["model"], "auto");
    assert_eq!(payload["stream"], false);
    assert_eq!(payload["max_tokens"], 1);
    Ok(())
}

async fn vllm_sr_chat_probe(
    State(requests): State<Arc<Mutex<Vec<Value>>>>,
    body: Bytes,
) -> Response {
    let payload = serde_json::from_slice::<Value>(&body)
        .unwrap_or_else(|error| Value::String(format!("invalid_json:{error}")));
    requests
        .lock()
        .expect("request capture lock should not be poisoned")
        .push(payload);

    let mut response = (StatusCode::OK, r#"{"model":"fallback-model"}"#).into_response();
    response.headers_mut().insert(
        VLLM_SR_SELECTED_DECISION_HEADER,
        HeaderValue::from_static("audio-decision"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_MODEL_HEADER,
        HeaderValue::from_static("qwen/qwen3-asr"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_CONFIDENCE_HEADER,
        HeaderValue::from_static("0.99"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_REASONING_HEADER,
        HeaderValue::from_static("off"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_MODALITY_HEADER,
        HeaderValue::from_static("AR"),
    );
    response
}
