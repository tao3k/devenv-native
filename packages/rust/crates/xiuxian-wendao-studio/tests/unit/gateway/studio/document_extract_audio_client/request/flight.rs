use std::sync::{Arc, Mutex};

use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightRequestOptions,
};
use crate::unit::gateway::studio::document_extract_audio_client::support::{
    ObservedAudioShardWindow, sample_input, spawn_audio_shard_service,
};
use xiuxian_io::model_routing::{WendaoModelDecision, WendaoRouteIntent};
use xiuxian_wendao_attachments::audio::{AudioShardResult, build_audio_shard_result_batch};

#[tokio::test]
async fn audio_shard_flight_client_roundtrips_results() -> Result<(), String> {
    let input = sample_input();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    assert_eq!(client.endpoint_url(), endpoint);
    let response = client.request(std::slice::from_ref(&input)).await?;

    assert_eq!(response.results, vec![success]);
    let merge_report = response.merge_for_inputs(std::slice::from_ref(&input))?;
    assert_eq!(merge_report.text, "audio text");
    assert!(merge_report.has_complete_success_coverage());
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.descriptor_path, vec!["analysis", "audio-shards"]);
    assert_eq!(observed.row_count, 1);
    assert_eq!(observed.sample_rate_hz, 16_000);
    assert_eq!(observed.start_ms, 0);
    assert_eq!(observed.duration_ms, 30_000);
    assert_eq!(observed.media_start_ms, 0);
    assert_eq!(observed.media_duration_ms, 30_000);
    assert_eq!(observed.source_path, "/tmp/source.mp3");
    assert_eq!(observed.backend_profile, "hosted-audio");
    assert_eq!(observed.worker_budget_header, None);

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_preserves_variable_window_rows() -> Result<(), String> {
    let mut input = sample_input();
    input.start_ms = 9_000;
    input.duration_ms = 8_000;
    input.media_start_ms = 8_500;
    input.media_duration_ms = 9_200;
    input.reading_order_key = "000001.000000009000".to_owned();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client.request(std::slice::from_ref(&input)).await?;

    assert_eq!(response.results, vec![success]);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.start_ms, 9_000);
    assert_eq!(observed.duration_ms, 8_000);
    assert_eq!(observed.media_start_ms, 8_500);
    assert_eq!(observed.media_duration_ms, 9_200);
    assert_eq!(
        observed.windows,
        vec![ObservedAudioShardWindow {
            start_ms: 9_000,
            duration_ms: 8_000,
            media_start_ms: 8_500,
            media_duration_ms: 9_200,
            reading_order_key: "000001.000000009000".to_owned(),
        }]
    );

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_sends_worker_budget_header() -> Result<(), String> {
    let input = sample_input();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client
        .request_with_worker_budget(std::slice::from_ref(&input), Some(4))
        .await?;

    assert_eq!(response.results, vec![success]);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.worker_budget_header.as_deref(), Some("4"));

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_sends_backend_selection_headers() -> Result<(), String> {
    let input = sample_input();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client
        .request_with_options(
            std::slice::from_ref(&input),
            &AudioShardFlightRequestOptions {
                audio_worker: Some("hosted".to_owned()),
                hosted_provider: Some("openrouter".to_owned()),
                hosted_base_url: Some("https://openrouter.ai/api/v1".to_owned()),
                hosted_endpoint: Some("audio-transcriptions".to_owned()),
                hosted_model: Some("xiaomi/mimo-v2.5".to_owned()),
                ..AudioShardFlightRequestOptions::default()
            },
        )
        .await?;

    assert_eq!(response.results, vec![success]);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.audio_worker_header.as_deref(), Some("hosted"));
    assert_eq!(
        observed.hosted_provider_header.as_deref(),
        Some("openrouter")
    );
    assert_eq!(
        observed.hosted_base_url_header.as_deref(),
        Some("https://openrouter.ai/api/v1")
    );
    assert_eq!(
        observed.hosted_endpoint_header.as_deref(),
        Some("audio-transcriptions")
    );
    assert_eq!(
        observed.hosted_model_header.as_deref(),
        Some("xiaomi/mimo-v2.5")
    );

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_sends_model_route_decision_headers() -> Result<(), String> {
    let input = sample_input();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client
        .request_with_options(
            std::slice::from_ref(&input),
            &AudioShardFlightRequestOptions {
                route_intent: Some(WendaoRouteIntent {
                    task_kind: "attachment-extract".into(),
                    modality: "audio".to_owned(),
                    source_kind: "attachment".into(),
                    precision_tier: "high".to_owned(),
                    privacy_tier: "private".to_owned(),
                    latency_budget_ms: 120_000,
                    evidence_profile: "transcript".to_owned(),
                    artifact_refs: vec!["artifact://audio/001".to_owned()],
                }),
                model_decision: Some(WendaoModelDecision {
                    route_id: "route-audio-1".to_owned(),
                    selected_provider: "openrouter".to_owned(),
                    selected_model: "qwen/qwen3-asr-flash-2026-02-10".to_owned(),
                    selected_backend_profile: "hosted-audio-transcript-v1".to_owned(),
                    reasoning_policy: Some("none".to_owned()),
                    route_trace: Some("matched audio card".to_owned()),
                }),
                ..AudioShardFlightRequestOptions::default()
            },
        )
        .await?;

    assert_eq!(response.results, vec![success]);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.route_id_header.as_deref(), Some("route-audio-1"));
    assert_eq!(
        observed.route_task_kind_header.as_deref(),
        Some("attachment-extract")
    );
    assert_eq!(observed.route_modality_header.as_deref(), Some("audio"));
    assert_eq!(
        observed.route_selected_provider_header.as_deref(),
        Some("openrouter")
    );
    assert_eq!(
        observed.route_selected_model_header.as_deref(),
        Some("qwen/qwen3-asr-flash-2026-02-10")
    );
    assert_eq!(
        observed.route_selected_backend_profile_header.as_deref(),
        Some("hosted-audio-transcript-v1")
    );
    assert_eq!(
        observed.route_precision_tier_header.as_deref(),
        Some("high")
    );

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_rejects_empty_input() -> Result<(), String> {
    let input = sample_input();
    let response_batch =
        build_audio_shard_result_batch(&[AudioShardResult::skipped(&input, "unused")])?;
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::new(Mutex::new(None))).await?;
    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;

    let Err(error) = client.request(&[]).await else {
        return Err("empty input should be rejected".to_owned());
    };

    assert_eq!(error, "audio shard request inputs cannot be empty");
    server_handle.abort();
    Ok(())
}
