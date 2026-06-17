use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::Value;
use tokio::net::TcpListener;
use xiuxian_io::model_routing::{
    VLLM_SR_SELECTED_DECISION_HEADER, VLLM_SR_SELECTED_MODEL_HEADER, WendaoChatRouteConfig,
    WendaoModelRoutingMode,
};

use crate::studio::router::handlers::model_route::{
    ChatModelRouteRequest, admit_chat_model_route_with_config,
};

#[tokio::test]
async fn chat_model_route_admission_uses_gateway_vllm_sr_decision() -> Result<(), String> {
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

    let response = admit_chat_model_route_with_config(
        WendaoChatRouteConfig {
            route_provider: Some("openrouter".to_owned()),
            route_model: "deepseek/deepseek-v4-pro".to_owned(),
            backend_profile: "openai-compatible-chat-v1".to_owned(),
            model_routing_mode: WendaoModelRoutingMode::VllmSr,
            vllm_sr_base_url: format!("http://{addr}"),
        },
        ChatModelRouteRequest {
            precision_tier: "high".to_owned(),
            privacy_tier: "private".to_owned(),
            latency_budget_ms: 45_000,
            evidence_profile: "local-knowledge-chat".to_owned(),
            artifact_refs: vec!["artifact://evidence-pack/001".to_owned()],
        },
    )
    .await
    .map_err(|error| error.error.message)?;

    let decision = response.decision;
    assert_eq!(
        response.schema_version,
        "xiuxian_wendao.model_route_chat_admission.v1"
    );
    assert_eq!(response.model_routing_mode, "vllm-sr");
    assert_eq!(response.intent.task_kind.as_str(), "chat");
    assert_eq!(response.intent.modality, "text");
    assert_eq!(response.intent.latency_budget_ms, 45_000);
    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "deepseek/deepseek-v4-pro");
    assert_eq!(
        decision.selected_backend_profile,
        "openai-compatible-chat-v1"
    );

    let captured = requests
        .lock()
        .map_err(|_| "request capture lock poisoned".to_owned())?;
    let payload = captured
        .first()
        .ok_or_else(|| "expected vLLM-SR chat route probe".to_owned())?;
    assert_eq!(payload["model"], "auto");
    assert_eq!(payload["stream"], false);
    assert_eq!(payload["max_tokens"], 1);
    Ok(())
}

#[tokio::test]
async fn chat_model_route_admission_allows_explicit_deterministic_mode() -> Result<(), String> {
    let response = admit_chat_model_route_with_config(
        WendaoChatRouteConfig {
            route_provider: None,
            route_model: "deepseek-chat".to_owned(),
            backend_profile: "openai-compatible-chat-v1".to_owned(),
            model_routing_mode: WendaoModelRoutingMode::Deterministic,
            vllm_sr_base_url: "http://127.0.0.1:8888".to_owned(),
        },
        ChatModelRouteRequest {
            precision_tier: String::new(),
            privacy_tier: String::new(),
            latency_budget_ms: 60_000,
            evidence_profile: String::new(),
            artifact_refs: Vec::new(),
        },
    )
    .await
    .map_err(|error| error.error.message)?;

    assert_eq!(response.model_routing_mode, "deterministic");
    assert_eq!(response.decision.selected_provider, "deepseek");
    assert_eq!(response.decision.selected_model, "deepseek-chat");
    assert_eq!(response.intent.task_kind.as_str(), "chat");
    assert_eq!(response.intent.precision_tier, "high");
    assert_eq!(response.intent.privacy_tier, "private");
    assert_eq!(response.intent.evidence_profile, "local-knowledge-chat");
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
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(payload);

    let mut response = (StatusCode::OK, r#"{"model":"fallback-model"}"#).into_response();
    response.headers_mut().insert(
        VLLM_SR_SELECTED_DECISION_HEADER,
        HeaderValue::from_static("chat-decision"),
    );
    response.headers_mut().insert(
        VLLM_SR_SELECTED_MODEL_HEADER,
        HeaderValue::from_static("deepseek/deepseek-v4-pro"),
    );
    response
}
