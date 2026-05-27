use std::sync::{Arc, Mutex};

use axum::http::{HeaderMap, HeaderValue};
use axum::{Json, Router, routing::post};
use serde_json::json;
use tokio::net::TcpListener;
use xiuxian_llm::model_routing::{
    VLLM_SR_AUTO_MODEL, VLLM_SR_REQUEST_ID_HEADER, VLLM_SR_SELECTED_DECISION_HEADER,
    VLLM_SR_SELECTED_MODEL_HEADER,
};
use xiuxian_wendao_attachments::audio::{AudioShardPlan, AudioSourceIdentity};
use xiuxian_wendao_server::transport::{DocumentExtractFlightRequest, DocumentExtractMode};

use super::audio_model_route_decision_for_document_extract;

#[test]
fn audio_route_normalizes_source_hash_identity() -> Result<(), String> {
    assert_eq!(
        super::normalized_source_hash(" sourcehash ")?,
        "sourcehash".to_owned()
    );
    assert!(super::normalized_source_hash("   ").is_err());
    Ok(())
}

#[tokio::test]
async fn audio_route_admission_uses_vllm_sr_decision() -> Result<(), String> {
    let observed_payload = Arc::new(Mutex::new(None));
    let (vllm_sr_base_url, server_handle) =
        spawn_vllm_sr_route_probe(Arc::clone(&observed_payload)).await?;
    let config = super::super::document_extract_audio_config(&|key| match key {
        "WENDAO_MODEL_ROUTING_MODE" => Some("vllm-sr".to_owned()),
        "WENDAO_VLLM_SR_BASE_URL" => Some(vllm_sr_base_url.clone()),
        "WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        _ => None,
    })?;
    let request = audio_route_request();
    let plan = audio_route_plan();

    let Some((intent, decision)) = audio_model_route_decision_for_document_extract(
        &request,
        &config,
        &plan,
        "sourcehash",
        61_000,
    )
    .await?
    else {
        return Err("vLLM-SR mode should produce a route decision".to_owned());
    };

    assert_eq!(decision.selected_provider, "openrouter");
    assert_eq!(decision.selected_model, "qwen/qwen3-asr-flash-2026-02-10");
    assert_eq!(
        decision.selected_backend_profile,
        "hosted-audio-transcript-v1"
    );
    assert_eq!(intent.modality, "audio");
    assert_eq!(intent.task_kind.as_str(), "attachment-extract");
    assert!(
        intent
            .artifact_refs
            .contains(&"source-sha256:sourcehash".to_owned())
    );
    assert!(intent.artifact_refs.contains(&"shard-count:2".to_owned()));
    let payload = observed_payload
        .lock()
        .map_err(|_| "observed payload lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "vLLM-SR probe did not receive a request".to_owned())?;
    assert_eq!(
        payload.get("model").and_then(serde_json::Value::as_str),
        Some(VLLM_SR_AUTO_MODEL)
    );

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_route_admission_requires_provider_hint() -> Result<(), String> {
    let config = super::super::document_extract_audio_config(&|key| match key {
        "WENDAO_MODEL_ROUTING_MODE" => Some("vllm-sr".to_owned()),
        "WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER" => Some(" ".to_owned()),
        _ => None,
    })?;
    let error = audio_model_route_decision_for_document_extract(
        &audio_route_request(),
        &config,
        &audio_route_plan(),
        "sourcehash",
        61_000,
    )
    .await
    .expect_err("vLLM-SR route admission should require a provider hint");

    assert!(error.contains("requires a route provider"));
    Ok(())
}

#[tokio::test]
async fn audio_route_deterministic_mode_returns_gateway_decision() -> Result<(), String> {
    let config = super::super::document_extract_audio_config(&|key| match key {
        "WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        "WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL" => Some("qwen/qwen3-asr-flash-2026-02-10".to_owned()),
        _ => None,
    })?;

    let Some((intent, decision)) = audio_model_route_decision_for_document_extract(
        &audio_route_request(),
        &config,
        &audio_route_plan(),
        "sourcehash",
        61_000,
    )
    .await?
    else {
        return Err("deterministic mode should produce a route decision".to_owned());
    };

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

async fn spawn_vllm_sr_route_probe(
    observed_payload: Arc<Mutex<Option<serde_json::Value>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("bind vLLM-SR route probe: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("read vLLM-SR route probe address: {error}"))?;
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |Json(payload): Json<serde_json::Value>| {
            let observed_payload = Arc::clone(&observed_payload);
            async move {
                *observed_payload
                    .lock()
                    .expect("observed payload lock should not be poisoned") = Some(payload);
                let mut headers = HeaderMap::new();
                headers.insert(
                    VLLM_SR_SELECTED_MODEL_HEADER,
                    HeaderValue::from_static("qwen/qwen3-asr-flash-2026-02-10"),
                );
                headers.insert(
                    VLLM_SR_SELECTED_DECISION_HEADER,
                    HeaderValue::from_static("audio-transcription"),
                );
                headers.insert(
                    VLLM_SR_REQUEST_ID_HEADER,
                    HeaderValue::from_static("route-audio-1"),
                );
                (
                    headers,
                    Json(json!({"model": "qwen/qwen3-asr-flash-2026-02-10"})),
                )
            }
        }),
    );
    let handle = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            panic!("vLLM-SR route probe failed: {error}");
        }
    });
    Ok((format!("http://{address}"), handle))
}

fn audio_route_request() -> DocumentExtractFlightRequest {
    DocumentExtractFlightRequest {
        source_path: "/tmp/source.mp3".to_owned(),
        output_dir: "/tmp/out".to_owned(),
        force: false,
        error_row: false,
        profile: "default".to_owned(),
        mode: DocumentExtractMode::AudioShards,
        wait_ms: 0,
        audio_worker: None,
        audio_hosted_provider: None,
        audio_hosted_base_url: None,
        audio_hosted_endpoint: None,
        audio_hosted_model: None,
    }
}

fn audio_route_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "source".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(61_000),
        },
        chunk_duration_ms: 30_000,
        start_offsets_ms: vec![0, 30_000],
        window_durations_ms: vec![30_000, 31_000],
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
        strategy: "full-coverage".to_owned(),
    }
}
