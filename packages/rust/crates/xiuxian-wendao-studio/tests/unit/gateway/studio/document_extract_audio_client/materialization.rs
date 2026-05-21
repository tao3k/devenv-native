use std::sync::{Arc, Mutex};

use super::support::{
    ObservedAudioShardWindow, error_to_string, make_executable, sample_materialized_item,
    sample_speech_window_planner_input, sample_variable_window_plan, spawn_audio_shard_service,
};
use crate::studio::document_extract_audio_client::AudioShardFlightClient;
use xiuxian_wendao_attachments::audio::{
    AudioShardMaterializationInput, AudioShardResult, AudioShardWorkerProfile,
    build_audio_shard_inputs, build_audio_shard_result_batch, build_audio_speech_window_plan,
    materialize_audio_shards,
};

#[tokio::test]
async fn audio_shard_flight_client_builds_inputs_from_materialized_shards() -> Result<(), String> {
    let materialized = sample_materialized_item();
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let expected_input = build_audio_shard_inputs(std::slice::from_ref(&materialized), &profile)
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard input".to_owned())?;
    let success = AudioShardResult::succeeded(&expected_input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client
        .request_materialized_with_worker_budget(
            std::slice::from_ref(&materialized),
            &profile,
            Some(2),
        )
        .await?;

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
    assert_eq!(observed.backend_profile, "hosted-audio-transcript-v1");
    assert_eq!(observed.worker_budget_header.as_deref(), Some("2"));

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_materializes_plan_before_exchange() -> Result<(), String> {
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
    let response = client
        .request_plan_with_worker_budget(&plan, &cached_materialization, &profile, Some(2))
        .await?;

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
    assert_eq!(observed.backend_profile, "hosted-audio-transcript-v1");
    assert_eq!(observed.worker_budget_header.as_deref(), Some("2"));

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_builds_speech_window_plan_before_exchange() -> Result<(), String>
{
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
    let planner_input = sample_speech_window_planner_input();
    let plan = build_audio_speech_window_plan(&planner_input)?;
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
        .nth(1)
        .ok_or_else(|| "expected second audio shard input".to_owned())?;
    let success = AudioShardResult::succeeded(&expected_input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client
        .request_speech_window_plan_with_worker_budget(
            &planner_input,
            &cached_materialization,
            &profile,
            Some(2),
        )
        .await?;

    assert_eq!(response.results, vec![success]);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.row_count, 2);
    assert_eq!(observed.start_ms, 0);
    assert_eq!(observed.duration_ms, 4_000);
    assert_eq!(observed.media_start_ms, 0);
    assert_eq!(observed.media_duration_ms, 4_700);
    assert_eq!(
        observed.windows,
        vec![
            ObservedAudioShardWindow {
                start_ms: 0,
                duration_ms: 4_000,
                media_start_ms: 0,
                media_duration_ms: 4_700,
                reading_order_key: "000000.000000000000".to_owned(),
            },
            ObservedAudioShardWindow {
                start_ms: 9_000,
                duration_ms: 8_000,
                media_start_ms: 8_500,
                media_duration_ms: 9_200,
                reading_order_key: "000001.000000009000".to_owned(),
            },
        ]
    );
    assert_eq!(observed.backend_profile, "hosted-audio-transcript-v1");
    assert_eq!(observed.worker_budget_header.as_deref(), Some("2"));

    server_handle.abort();
    Ok(())
}
