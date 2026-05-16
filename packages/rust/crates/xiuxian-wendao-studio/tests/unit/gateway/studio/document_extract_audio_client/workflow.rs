use std::sync::{Arc, Mutex};

use arrow::record_batch::RecordBatch as EngineRecordBatch;

use super::support::{
    error_to_string, make_executable, sample_variable_window_plan, spawn_audio_shard_service,
};
use crate::studio::document_extract_audio_client::AudioShardFlightClient;
use xiuxian_wendao_attachments::audio::{
    AudioShardMaterializationInput, AudioShardResult, AudioShardWorkerProfile,
    build_audio_shard_inputs, build_audio_shard_result_batch, materialize_audio_shards,
};

#[tokio::test]
async fn audio_shard_flight_client_executes_plan_as_typed_workflow() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    std::fs::write(
        ffmpeg_path.as_path(),
        "#!/bin/sh\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\nprintf cached > \"$last\"\n",
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;
    let plan = sample_variable_window_plan();
    let materialization = AudioShardMaterializationInput {
        source_path,
        output_dir: tempdir.path().join("chunks"),
        ffmpeg_path,
        force: true,
    };
    let expected_materialized = materialize_audio_shards(&plan, &materialization)?;
    let cached_materialization = AudioShardMaterializationInput {
        force: false,
        ..materialization
    };
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let expected_input = build_audio_shard_inputs(expected_materialized.as_slice(), &profile)
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard input".to_owned())?;
    let success = AudioShardResult::succeeded(&expected_input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let execution = client
        .execute_plan_with_worker_budget(&plan, &cached_materialization, &profile, Some(2))
        .await?;

    assert_eq!(execution.plan, plan);
    assert_eq!(execution.materialized_shards.len(), 1);
    assert_eq!(execution.inputs, vec![expected_input]);
    assert_eq!(execution.response.results, vec![success]);
    assert_eq!(execution.merge_report.text, "audio text");
    assert!(execution.merge_report.has_complete_success_coverage());
    assert_eq!(
        execution
            .trace
            .stages
            .iter()
            .map(|stage| stage.stage_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "audio.materialize_shards",
            "audio.build_arrow_rows",
            "audio.call_analyzer_flight",
            "audio.merge_precision_gate",
        ]
    );
    assert_eq!(execution.trace.stages[0].input.item_count, Some(1));
    assert_eq!(execution.trace.stages[1].output.item_count, Some(1));
    assert_eq!(execution.trace.stages[2].output.item_count, Some(1));
    assert_eq!(
        execution.trace.stages[1]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["audio.arrow.input_batch.v1"]
    );
    assert_eq!(
        execution.trace.stages[2]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["audio.arrow.result_batches.v1"]
    );
    let input_batch = execution
        .memory_checkpoints
        .get::<EngineRecordBatch>("audio.arrow.input_batch.v1")
        .map_err(error_to_string)?;
    assert_eq!(input_batch.num_rows(), 1);
    let result_batches = execution
        .memory_checkpoints
        .get::<Vec<EngineRecordBatch>>("audio.arrow.result_batches.v1")
        .map_err(error_to_string)?;
    assert_eq!(result_batches.len(), 1);
    assert_eq!(result_batches[0].num_rows(), 1);

    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.worker_budget_header.as_deref(), Some("2"));

    server_handle.abort();
    Ok(())
}
