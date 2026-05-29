use std::sync::Arc;

use crate::model_routing::vllm_sr_chat_probe;
use axum::Router;
use axum::routing::post;
use std::sync::Mutex;
use tokio::net::TcpListener;
use xiuxian_llm::model_routing::{VllmSrRouteDecisionClient, WendaoRouteIntent};

#[tokio::test]
async fn vllm_sr_route_probe_parses_official_selection_headers() -> Result<(), String> {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| error.to_string())?;
    let addr = listener.local_addr().map_err(|error| error.to_string())?;
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(|state, body| async move { vllm_sr_chat_probe(state, &body) }),
        )
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
