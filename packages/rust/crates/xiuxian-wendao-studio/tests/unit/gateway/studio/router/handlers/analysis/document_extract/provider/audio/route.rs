use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use sha2::Digest;
use xiuxian_wendao_attachments::audio::{
    AudioRiskParentSelectionOptions, AudioShardInput, AudioShardMaterializationInput,
    AudioShardPlan, AudioShardResult, AudioShardWorkerProfile, build_audio_shard_inputs,
    build_audio_shard_result_batch, materialize_audio_shards,
};
use xiuxian_wendao_server::transport::{DocumentExtractFlightRequest, DocumentExtractMode};

use crate::studio::document_extract_audio_client::{
    AudioShardFlightResponse, AudioShardRecoveryPlanRequest,
};
use crate::studio::router::handlers::analysis::document_extract::provider::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::provider::audio::{
    build_full_coverage_audio_plan, document_extract_audio_config,
};
use crate::studio::router::handlers::analysis::document_extract::registry::DocumentExtractJobRegistry;
use crate::unit::gateway::studio::document_extract_audio_client::support::{
    ObservedAudioShardRequest, make_executable, spawn_audio_shard_sequence_service,
};

use super::string_column;

#[tokio::test]
async fn audio_shards_document_extract_batch_roundtrips_fake_flight() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source_path = temp.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(|error| error.to_string())?;
    let output_dir = temp.path().join("out");
    let (ffmpeg_path, ffprobe_path) = audio_route_fake_tools(&temp)?;
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS" => Some("30000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS" => Some("15000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFMPEG" => Some(ffmpeg_path.to_string_lossy().to_string()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFPROBE" => Some(ffprobe_path.to_string_lossy().to_string()),
        _ => None,
    })?;
    let source_hash = format!("{:x}", sha2::Sha256::digest(b"source"));
    let plan = build_full_coverage_audio_plan(
        source_path.as_path(),
        source_hash.clone(),
        61_000,
        &config,
    )?;
    let materialization = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: output_dir.join("audio_shards"),
        ffmpeg_path: ffmpeg_path.clone(),
        artifact_cache_dir: None,
        force: true,
    };
    let profile = AudioShardWorkerProfile::transcription(config.backend_profile.as_str());
    let shard_batches = audio_route_shard_batches(
        &plan,
        &materialization,
        &profile,
        config.recovery_split_duration_ms,
    )?;
    let changed_config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS" => Some("15000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS" => Some("15000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFMPEG" => Some(ffmpeg_path.to_string_lossy().to_string()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFPROBE" => Some(ffprobe_path.to_string_lossy().to_string()),
        _ => None,
    })?;
    let changed_plan = build_full_coverage_audio_plan(
        source_path.as_path(),
        source_hash,
        61_000,
        &changed_config,
    )?;
    let changed_profile =
        AudioShardWorkerProfile::transcription(changed_config.backend_profile.as_str());
    let changed_shard_batches = audio_route_shard_batches(
        &changed_plan,
        &materialization,
        &changed_profile,
        changed_config.recovery_split_duration_ms,
    )?;
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![
            shard_batches.response_batch.clone(),
            shard_batches.recovery_response_batch.clone(),
            changed_shard_batches.response_batch.clone(),
            changed_shard_batches.recovery_response_batch.clone(),
        ],
        Arc::clone(&observed),
        Arc::clone(&observed_requests),
    )
    .await?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider =
        StudioDocumentExtractFlightRouteProvider::from_registry_with_document_extract_endpoint(
            Ok(registry),
            1,
            endpoint,
        );

    let response = provider
        .audio_shards_document_extract_batch_with_config(
            &DocumentExtractFlightRequest {
                source_path: source_path.to_string_lossy().to_string(),
                output_dir: output_dir.to_string_lossy().to_string(),
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
            },
            config.clone(),
        )
        .await?;

    assert_audio_route_observed_requests(
        &observed,
        &observed_requests,
        shard_batches.recovery_inputs.len(),
    )?;
    let batch = response
        .batches
        .first()
        .ok_or_else(|| "expected document resource response batch".to_owned())?;
    assert_audio_route_resource_batch(batch)?;
    assert_audio_route_manifest_chunk_ms(&output_dir, 30_000)?;

    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(source_path.as_path(), b"source").map_err(|error| error.to_string())?;

    let cached_response = provider
        .audio_shards_document_extract_batch_with_config(
            &DocumentExtractFlightRequest {
                source_path: source_path.to_string_lossy().to_string(),
                output_dir: output_dir.to_string_lossy().to_string(),
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
            },
            config,
        )
        .await?;
    let cached_batch = cached_response
        .batches
        .first()
        .ok_or_else(|| "expected cached document resource response batch".to_owned())?;
    assert_audio_route_cached_batch(batch, cached_batch)?;
    let observed_request_rows = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?;
    assert_eq!(observed_request_rows.len(), 2);
    drop(observed_request_rows);

    let changed_response = provider
        .audio_shards_document_extract_batch_with_config(
            &DocumentExtractFlightRequest {
                source_path: source_path.to_string_lossy().to_string(),
                output_dir: output_dir.to_string_lossy().to_string(),
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
            },
            changed_config,
        )
        .await?;
    let changed_batch = changed_response
        .batches
        .first()
        .ok_or_else(|| "expected changed document resource response batch".to_owned())?;
    assert_audio_route_resource_batch(changed_batch)?;
    assert_audio_route_manifest_chunk_ms(&output_dir, 15_000)?;
    let observed_request_rows = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?;
    assert!(observed_request_rows.len() >= 3);
    assert_eq!(
        observed_request_rows[2].row_count,
        changed_plan.start_offsets_ms.len()
    );

    server_handle.abort();
    Ok(())
}

struct AudioRouteShardBatches {
    recovery_inputs: Vec<AudioShardInput>,
    response_batch: EngineRecordBatch,
    recovery_response_batch: EngineRecordBatch,
}

fn audio_route_fake_tools(temp: &tempfile::TempDir) -> Result<(PathBuf, PathBuf), String> {
    let ffmpeg_path = temp.path().join("fake_ffmpeg.sh");
    std::fs::write(
        ffmpeg_path.as_path(),
        "#!/bin/sh\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\nprintf cached > \"$last\"\n",
    )
    .map_err(|error| error.to_string())?;
    make_executable(ffmpeg_path.as_path())?;
    let ffprobe_path = temp.path().join("fake_ffprobe.sh");
    std::fs::write(ffprobe_path.as_path(), "#!/bin/sh\nprintf '61.0\\n'\n")
        .map_err(|error| error.to_string())?;
    make_executable(ffprobe_path.as_path())?;

    Ok((ffmpeg_path, ffprobe_path))
}

fn audio_route_shard_batches(
    plan: &AudioShardPlan,
    materialization: &AudioShardMaterializationInput,
    profile: &AudioShardWorkerProfile,
    split_duration_ms: u64,
) -> Result<AudioRouteShardBatches, String> {
    let materialized = materialize_audio_shards(plan, materialization)?;
    let inputs = build_audio_shard_inputs(materialized.as_slice(), profile);
    let results = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            AudioShardResult::succeeded(input, format!("chunk {index} text"), 0.99)
        })
        .collect::<Vec<_>>();
    let recovery_planning = AudioShardFlightResponse {
        results: results.clone(),
    }
    .plan_recovery_split(AudioShardRecoveryPlanRequest {
        parent_plan: plan,
        inputs: inputs.as_slice(),
        request_metrics: &[],
        selection_options: AudioRiskParentSelectionOptions::default(),
        split_duration_ms,
        speech_window_input: None,
    })?;
    let recovery_materialized =
        materialize_audio_shards(&recovery_planning.recovery_plan, materialization)?;
    let recovery_inputs = build_audio_shard_inputs(recovery_materialized.as_slice(), profile);
    let recovery_results = recovery_inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            AudioShardResult::succeeded(input, format!("recovery {index} text"), 0.99)
        })
        .collect::<Vec<_>>();

    Ok(AudioRouteShardBatches {
        recovery_inputs,
        response_batch: build_audio_shard_result_batch(results.as_slice())?,
        recovery_response_batch: build_audio_shard_result_batch(recovery_results.as_slice())?,
    })
}

fn assert_audio_route_observed_requests(
    observed: &Arc<Mutex<Option<ObservedAudioShardRequest>>>,
    observed_requests: &Arc<Mutex<Vec<ObservedAudioShardRequest>>>,
    recovery_input_count: usize,
) -> Result<(), String> {
    let observed_request_rows = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?
        .clone();
    assert_eq!(observed_request_rows.len(), 2);
    assert_eq!(
        observed_request_rows[0].descriptor_path,
        vec!["analysis", "audio-shards"]
    );
    assert_eq!(observed_request_rows[0].row_count, 3);
    assert_eq!(observed_request_rows[1].row_count, recovery_input_count);
    let base_budget = observed_request_rows[0]
        .worker_budget_header
        .as_deref()
        .ok_or_else(|| "base audio request should carry Rust-selected worker budget".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("base worker budget header should be numeric: {error}"))?;
    assert!((1..=3).contains(&base_budget));
    let recovery_budget = observed_request_rows[1]
        .worker_budget_header
        .as_deref()
        .ok_or_else(|| {
            "recovery audio request should carry Rust-selected worker budget".to_owned()
        })?
        .parse::<usize>()
        .map_err(|error| format!("recovery worker budget header should be numeric: {error}"))?;
    assert!(recovery_budget >= 1);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "fake analyzer did not receive audio shards".to_owned())?;
    assert_eq!(observed.descriptor_path, vec!["analysis", "audio-shards"]);
    assert_eq!(observed.row_count, recovery_input_count);
    assert_eq!(observed.backend_profile, "hosted-audio-transcript-v1");
    Ok(())
}

fn assert_audio_route_resource_batch(batch: &EngineRecordBatch) -> Result<(), String> {
    assert_eq!(batch.num_rows(), 2);
    assert_eq!(
        string_column(batch, "resourceType")?.value(0),
        "audio-transcript"
    );
    assert_eq!(
        string_column(batch, "resourceType")?.value(1),
        "audio-transcript-ledger"
    );
    assert_eq!(string_column(batch, "mimeType")?.value(0), "text/plain");
    assert_eq!(string_column(batch, "mimeType")?.value(1), "text/org");
    assert!(!string_column(batch, "content")?.value(0).trim().is_empty());
    assert!(
        string_column(batch, "content")?
            .value(1)
            .contains("[[attachment:")
    );
    Ok(())
}

fn assert_audio_route_cached_batch(
    batch: &EngineRecordBatch,
    cached_batch: &EngineRecordBatch,
) -> Result<(), String> {
    assert_eq!(cached_batch.num_rows(), 2);
    assert_eq!(
        string_column(cached_batch, "content")?.value(0),
        string_column(batch, "content")?.value(0)
    );
    assert_eq!(
        string_column(cached_batch, "content")?.value(1),
        string_column(batch, "content")?.value(1)
    );
    Ok(())
}

fn assert_audio_route_manifest_chunk_ms(
    output_dir: &std::path::Path,
    expected_chunk_ms: u64,
) -> Result<(), String> {
    let manifest_path = output_dir.join("_audio_extract_manifest.json");
    let manifest = std::fs::read_to_string(manifest_path.as_path())
        .map_err(|error| format!("read audio cache manifest: {error}"))?;
    let value = serde_json::from_str::<serde_json::Value>(manifest.as_str())
        .map_err(|error| format!("parse audio cache manifest: {error}"))?;
    assert_eq!(
        value
            .get("chunkDurationMs")
            .and_then(serde_json::Value::as_u64),
        Some(expected_chunk_ms)
    );
    Ok(())
}
