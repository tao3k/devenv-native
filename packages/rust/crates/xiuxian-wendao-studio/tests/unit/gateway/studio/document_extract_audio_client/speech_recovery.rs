use std::sync::{Arc, Mutex};

use super::support::{
    error_to_string, make_executable, sample_input, spawn_audio_shard_sequence_service,
};
use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightResponse, AudioShardRecoveryPlanRequest,
    AudioShardRecoveryWorkflowRequest,
};
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions, AudioShardMaterializationInput,
    AudioShardPlan, AudioShardResult, AudioShardWorkerProfile, AudioSourceIdentity,
    AudioSpeechSegment, AudioSpeechWindowPlannerInput, build_audio_shard_inputs,
    build_audio_shard_result_batch, materialize_audio_shards,
};

#[tokio::test]
async fn audio_shard_response_plans_recovery_from_speech_timing_facts() -> Result<(), String> {
    let parent_plan = AudioShardPlan {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(180_000),
        },
        chunk_duration_ms: 60_000,
        start_offsets_ms: vec![0, 60_000, 120_000],
        window_durations_ms: Vec::new(),
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
        strategy: "full-coverage".to_owned(),
    };
    let mut selected = sample_input();
    selected.shard_element_id = "selected".to_owned();
    selected.start_ms = 60_000;
    selected.duration_ms = 60_000;
    selected.reading_order_key = "000001.000000060000".to_owned();
    let response = AudioShardFlightResponse {
        results: vec![AudioShardResult::failed(
            &selected,
            "audio transcript quality gate failed",
        )],
    };
    let speech_input = AudioSpeechWindowPlannerInput {
        profile: "audio-shards-v1".to_owned(),
        source: parent_plan.source.clone(),
        chunk_duration_ms: 15_000,
        limit_chunks: 8,
        speech_segments: vec![
            AudioSpeechSegment {
                index: 0,
                start_ms: 55_000,
                duration_ms: 10_000,
            },
            AudioSpeechSegment {
                index: 1,
                start_ms: 90_000,
                duration_ms: 4_000,
            },
            AudioSpeechSegment {
                index: 2,
                start_ms: 118_000,
                duration_ms: 8_000,
            },
        ],
        merge_gap_ms: 500,
        min_window_ms: 0,
        short_merge_gap_ms: Some(500),
        max_window_ms: Some(15_000),
        boundary_snap_tolerance_ms: 0,
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
    };

    let planning = response.plan_recovery_split(AudioShardRecoveryPlanRequest {
        parent_plan: &parent_plan,
        inputs: std::slice::from_ref(&selected),
        request_metrics: &[],
        selection_options: AudioRiskParentSelectionOptions {
            include_boundaries: false,
            ..Default::default()
        },
        split_duration_ms: 30_000,
        speech_window_input: Some(&speech_input),
    })?;

    assert_eq!(planning.selected_parent_inputs.len(), 1);
    assert_eq!(planning.recovery_plan.strategy, "speech-segments");
    assert_eq!(
        planning.recovery_plan.start_offsets_ms,
        vec![60_000, 90_000, 118_000]
    );
    assert_eq!(
        planning.recovery_plan.window_durations_ms,
        vec![5_000, 4_000, 2_000]
    );
    Ok(())
}

#[tokio::test]
async fn audio_shard_client_skips_recovery_when_speech_facts_miss_failed_parent()
-> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let parent_plan = missing_speech_parent_audio_plan();
    let materialization = missing_speech_materialization(&tempdir)?;
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let base_materialized = materialize_audio_shards(&parent_plan, &materialization)?;
    let base_inputs = build_audio_shard_inputs(base_materialized.as_slice(), &profile);
    let base_results = vec![
        AudioShardResult::failed(&base_inputs[0], "audio transcript quality gate failed"),
        AudioShardResult::succeeded(&base_inputs[1], "结束总结", 0.90),
    ];
    let base_batch = build_audio_shard_result_batch(base_results.as_slice())?;
    let cached_materialization = AudioShardMaterializationInput {
        force: false,
        ..materialization
    };
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![base_batch],
        Arc::clone(&observed),
        Arc::clone(&observed_requests),
    )
    .await?;
    let speech_input = AudioSpeechWindowPlannerInput {
        profile: "audio-shards-v1".to_owned(),
        source: parent_plan.source.clone(),
        chunk_duration_ms: 15_000,
        limit_chunks: 8,
        speech_segments: vec![AudioSpeechSegment {
            index: 0,
            start_ms: 90_000,
            duration_ms: 5_000,
        }],
        merge_gap_ms: 500,
        min_window_ms: 0,
        short_merge_gap_ms: Some(500),
        max_window_ms: Some(15_000),
        boundary_snap_tolerance_ms: 0,
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
    };

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let execution = client
        .execute_recovery_split(AudioShardRecoveryWorkflowRequest {
            parent_plan: &parent_plan,
            materialization: &cached_materialization,
            profile: &profile,
            request_metrics: &[],
            selection_options: AudioRiskParentSelectionOptions {
                include_boundaries: false,
                max_chars_per_minute: 0.0,
                ..Default::default()
            },
            patch_options: AudioRecoveryPatchGateOptions::default(),
            recovery_split_duration_ms: 30_000,
            recovery_speech_window_input: Some(&speech_input),
            base_worker_budget: Some(2),
            recovery_worker_budget: Some(1),
        })
        .await?;

    assert_eq!(execution.recovery_planning.selected_parent_inputs.len(), 1);
    assert_eq!(
        execution.recovery_planning.recovery_plan.strategy,
        "speech-window-recovery-empty"
    );
    assert!(execution.recovery_inputs.is_empty());
    assert!(execution.recovery_response.is_none());
    assert_eq!(execution.merge_report.failed_count, 1);
    assert!(!execution.merge_report.has_complete_success_coverage());
    assert_eq!(
        execution
            .trace
            .stages
            .iter()
            .map(|stage| stage.stage_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "audio.base.materialize_shards",
            "audio.base.build_arrow_rows",
            "audio.base.call_analyzer_flight",
            "audio.recovery.plan_split",
            "audio.recovery.merge_precision_gate",
        ]
    );
    let observed_requests = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?
        .clone();
    assert_eq!(observed_requests.len(), 1);
    assert_eq!(observed_requests[0].row_count, 2);

    server_handle.abort();
    Ok(())
}

fn missing_speech_parent_audio_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(120_000),
        },
        chunk_duration_ms: 60_000,
        start_offsets_ms: vec![0, 60_000],
        window_durations_ms: Vec::new(),
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
        strategy: "full-coverage".to_owned(),
    }
}

fn missing_speech_materialization(
    tempdir: &tempfile::TempDir,
) -> Result<AudioShardMaterializationInput, String> {
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    std::fs::write(
        ffmpeg_path.as_path(),
        "#!/bin/sh\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\nprintf cached > \"$last\"\n",
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;

    Ok(AudioShardMaterializationInput {
        source_path,
        output_dir: tempdir.path().join("chunks"),
        ffmpeg_path,
        artifact_cache_dir: None,
        force: true,
    })
}
