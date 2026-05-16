use std::sync::{Arc, Mutex};

use sha2::Digest;
use xiuxian_wendao_attachments::audio::{
    AudioRiskParentSelectionOptions, AudioShardMaterializationInput, AudioShardResult,
    AudioShardWorkerProfile, build_audio_shard_inputs, build_audio_shard_result_batch,
    materialize_audio_shards,
};
use xiuxian_wendao_server::transport::{DocumentExtractFlightRequest, DocumentExtractMode};

use crate::studio::document_extract_audio_client::tests::support::{
    make_executable, spawn_audio_shard_sequence_service,
};
use crate::studio::document_extract_audio_client::{
    AudioShardFlightResponse, AudioShardRecoveryPlanRequest,
};
use crate::studio::router::handlers::analysis::document_extract::provider::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::provider::audio::{
    build_full_coverage_audio_plan, document_extract_audio_config,
};
use crate::studio::router::handlers::analysis::document_extract::registry::DocumentExtractJobRegistry;

use super::string_column;

#[tokio::test]
async fn audio_shards_document_extract_batch_roundtrips_fake_flight() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source_path = temp.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(|error| error.to_string())?;
    let output_dir = temp.path().join("out");
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
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS" => Some("30000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS" => Some("15000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFMPEG" => Some(ffmpeg_path.to_string_lossy().to_string()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFPROBE" => Some(ffprobe_path.to_string_lossy().to_string()),
        _ => None,
    })?;
    let source_hash = format!("{:x}", sha2::Sha256::digest(b"source"));
    let plan = build_full_coverage_audio_plan(source_path.as_path(), source_hash, 61_000, &config)?;
    let materialization = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: output_dir.join("audio_shards"),
        ffmpeg_path: ffmpeg_path.clone(),
        force: true,
    };
    let materialized = materialize_audio_shards(&plan, &materialization)?;
    let profile = AudioShardWorkerProfile::transcription(config.backend_profile.as_str());
    let inputs = build_audio_shard_inputs(materialized.as_slice(), &profile);
    let results = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let text = format!("chunk {index} text");
            AudioShardResult::succeeded(input, text.as_str(), 0.99)
        })
        .collect::<Vec<_>>();
    let base_response = AudioShardFlightResponse {
        results: results.clone(),
    };
    let recovery_planning = base_response.plan_recovery_split(AudioShardRecoveryPlanRequest {
        parent_plan: &plan,
        inputs: inputs.as_slice(),
        request_metrics: &[],
        selection_options: AudioRiskParentSelectionOptions::default(),
        split_duration_ms: config.recovery_split_duration_ms,
        speech_window_input: None,
    })?;
    let recovery_materialized =
        materialize_audio_shards(&recovery_planning.recovery_plan, &materialization)?;
    let recovery_inputs = build_audio_shard_inputs(recovery_materialized.as_slice(), &profile);
    let recovery_results = recovery_inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let text = format!("recovery {index} text");
            AudioShardResult::succeeded(input, text.as_str(), 0.99)
        })
        .collect::<Vec<_>>();
    let response_batch = build_audio_shard_result_batch(results.as_slice())?;
    let recovery_response_batch = build_audio_shard_result_batch(recovery_results.as_slice())?;
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![response_batch, recovery_response_batch],
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
            },
            config.clone(),
        )
        .await?;

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
    assert_eq!(observed_request_rows[1].row_count, recovery_inputs.len());
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "fake analyzer did not receive audio shards".to_owned())?;
    assert_eq!(observed.descriptor_path, vec!["analysis", "audio-shards"]);
    assert_eq!(observed.row_count, recovery_inputs.len());
    assert_eq!(observed.backend_profile, "hosted-audio-transcript-v1");
    let batch = response
        .batches
        .first()
        .ok_or_else(|| "expected document resource response batch".to_owned())?;
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        string_column(batch, "resourceType")?.value(0),
        "audio-transcript"
    );
    assert_eq!(string_column(batch, "mimeType")?.value(0), "text/plain");
    assert!(!string_column(batch, "content")?.value(0).trim().is_empty());

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
            },
            config,
        )
        .await?;
    let cached_batch = cached_response
        .batches
        .first()
        .ok_or_else(|| "expected cached document resource response batch".to_owned())?;
    assert_eq!(cached_batch.num_rows(), 1);
    assert_eq!(
        string_column(cached_batch, "content")?.value(0),
        string_column(batch, "content")?.value(0)
    );
    let observed_requests = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?;
    assert_eq!(observed_requests.len(), 2);

    server_handle.abort();
    Ok(())
}
