use std::sync::{Arc, Mutex};

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_qianji::workflow_kernel::WorkflowCheckpointId;

use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightRequestOptions, AudioShardRecoveryWorkflowExecution,
    AudioShardRecoveryWorkflowRequest,
};
use crate::unit::gateway::studio::document_extract_audio_client::support::{
    ObservedAudioShardRequest, ObservedAudioShardWindow, error_to_string, make_executable,
    spawn_audio_shard_sequence_service,
};
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions, AudioShardInput,
    AudioShardMaterializationInput, AudioShardPlan, AudioShardRequestMetric, AudioShardResult,
    AudioShardWorkerProfile, AudioSourceIdentity, build_audio_recovery_split_plan,
    build_audio_shard_inputs, build_audio_shard_result_batch, materialize_audio_shards,
};

#[tokio::test]
async fn audio_shard_client_executes_two_pass_recovery_split() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let parent_plan = two_pass_parent_audio_plan();
    let materialization = two_pass_materialization(&tempdir)?;
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let base_batch = two_pass_base_batch(&parent_plan, &materialization, &profile)?;

    let recovery_plan = build_audio_recovery_split_plan(&parent_plan, &[1], 30_000)?;
    let recovery_batch = two_pass_recovery_batch(&recovery_plan, &materialization, &profile)?;
    let cached_materialization = AudioShardMaterializationInput {
        force: false,
        ..materialization
    };
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![
            base_batch.response_batch.clone(),
            recovery_batch.response_batch.clone(),
        ],
        Arc::clone(&observed),
        Arc::clone(&observed_requests),
    )
    .await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let execution = client
        .execute_recovery_split(AudioShardRecoveryWorkflowRequest {
            parent_plan: &parent_plan,
            materialization: &cached_materialization,
            profile: &profile,
            request_metrics: &[AudioShardRequestMetric {
                shard_element_id: base_batch.inputs[1].shard_element_id.clone(),
                wall_ms: 60_000,
            }],
            selection_options: AudioRiskParentSelectionOptions {
                include_boundaries: false,
                max_chars_per_minute: 0.0,
                max_chinese_ratio: 0.0,
                ..Default::default()
            },
            patch_options: AudioRecoveryPatchGateOptions::default(),
            recovery_split_duration_ms: 30_000,
            recovery_speech_window_input: None,
            base_worker_budget: Some(2),
            recovery_worker_budget: Some(1),
        })
        .await?;

    assert_two_pass_recovery_execution(&execution, &base_batch.results, &recovery_batch.results)?;
    assert_two_pass_trace(&execution)?;
    assert_two_pass_observed_requests(&observed_requests)?;

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_client_uses_planned_result_preflight_before_materialization()
-> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let parent_plan = preflight_parent_audio_plan();
    let materialization = preflight_materialization(&tempdir, "first", "fake_ffmpeg.sh")?;
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let materialized = materialize_audio_shards(&parent_plan, &materialization)?;
    let inputs = build_audio_shard_inputs(materialized.as_slice(), &profile);
    let result = AudioShardResult::succeeded(&inputs[0], "stable audio text", 0.98);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&result))?;
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![response_batch],
        Arc::clone(&observed),
        Arc::clone(&observed_requests),
    )
    .await?;
    let request_options = AudioShardFlightRequestOptions {
        audio_worker: Some("hosted".to_owned()),
        hosted_provider: Some("openrouter".to_owned()),
        hosted_model: Some("qwen/qwen3-asr-flash-2026-02-10".to_owned()),
        transcript_admission_dir: Some(tempdir.path().join("transcript-admission")),
        ..AudioShardFlightRequestOptions::default()
    };

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let first_execution = client
        .execute_recovery_split_with_options(
            AudioShardRecoveryWorkflowRequest {
                parent_plan: &parent_plan,
                materialization: &materialization,
                profile: &profile,
                request_metrics: &[],
                selection_options: AudioRiskParentSelectionOptions {
                    include_boundaries: false,
                    max_chars_per_minute: -1.0,
                    max_chinese_ratio: -1.0,
                    min_latency_ms: u64::MAX,
                    min_repeated_ngram_ratio: 2.0,
                    ..AudioRiskParentSelectionOptions::default()
                },
                patch_options: AudioRecoveryPatchGateOptions::default(),
                recovery_split_duration_ms: 30_000,
                recovery_speech_window_input: None,
                base_worker_budget: Some(1),
                recovery_worker_budget: Some(1),
            },
            request_options.clone(),
        )
        .await?;
    assert_eq!(
        first_execution
            .transcript_admission_stats()
            .planned_stored_count,
        1
    );

    let second_materialization = preflight_materialization(&tempdir, "second", "missing_ffmpeg")?;
    let second_execution = client
        .execute_recovery_split_with_options(
            AudioShardRecoveryWorkflowRequest {
                parent_plan: &parent_plan,
                materialization: &second_materialization,
                profile: &profile,
                request_metrics: &[],
                selection_options: AudioRiskParentSelectionOptions {
                    include_boundaries: false,
                    max_chars_per_minute: -1.0,
                    max_chinese_ratio: -1.0,
                    min_latency_ms: u64::MAX,
                    min_repeated_ngram_ratio: 2.0,
                    ..AudioRiskParentSelectionOptions::default()
                },
                patch_options: AudioRecoveryPatchGateOptions::default(),
                recovery_split_duration_ms: 30_000,
                recovery_speech_window_input: None,
                base_worker_budget: Some(1),
                recovery_worker_budget: Some(1),
            },
            request_options,
        )
        .await?;

    assert!(second_execution.base_materialized_shards.is_empty());
    assert_eq!(second_execution.base_inputs, inputs);
    assert_eq!(second_execution.base_response.results, vec![result]);
    assert_eq!(second_execution.transcript_admission_stats().hit_count, 1);
    assert_eq!(
        second_execution
            .transcript_admission_stats()
            .planned_hit_count,
        1
    );
    assert_eq!(second_execution.transcript_admission_stats().miss_count, 0);
    assert_eq!(second_execution.merge_report.text, "stable audio text");
    assert_eq!(
        observed_requests
            .lock()
            .map_err(|_| "observed request sequence lock poisoned".to_owned())?
            .len(),
        1
    );

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_client_materializes_only_partial_planned_admission_misses()
-> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let parent_plan = partial_preflight_parent_audio_plan();
    let seed_plan = partial_preflight_seed_audio_plan();
    let materialization = preflight_materialization(&tempdir, "partial", "partial_ffmpeg.sh")?;
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let seed_materialized = materialize_audio_shards(&seed_plan, &materialization)?;
    let seed_inputs = build_audio_shard_inputs(seed_materialized.as_slice(), &profile);
    let seed_result = AudioShardResult::succeeded(&seed_inputs[0], "first admitted text", 0.98);
    let parent_materialized = materialize_audio_shards(&parent_plan, &materialization)?;
    let parent_inputs = build_audio_shard_inputs(parent_materialized.as_slice(), &profile);
    let miss_result = AudioShardResult::succeeded(&parent_inputs[1], "second fresh text", 0.97);
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![
            build_audio_shard_result_batch(std::slice::from_ref(&seed_result))?,
            build_audio_shard_result_batch(std::slice::from_ref(&miss_result))?,
        ],
        Arc::clone(&observed),
        Arc::clone(&observed_requests),
    )
    .await?;
    let request_options = AudioShardFlightRequestOptions {
        audio_worker: Some("hosted".to_owned()),
        hosted_provider: Some("openrouter".to_owned()),
        hosted_model: Some("qwen/qwen3-asr-flash-2026-02-10".to_owned()),
        transcript_admission_dir: Some(tempdir.path().join("transcript-admission")),
        ..AudioShardFlightRequestOptions::default()
    };
    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    client
        .execute_recovery_split_with_options(
            AudioShardRecoveryWorkflowRequest {
                parent_plan: &seed_plan,
                materialization: &materialization,
                profile: &profile,
                request_metrics: &[],
                selection_options: no_recovery_selection_options(),
                patch_options: AudioRecoveryPatchGateOptions::default(),
                recovery_split_duration_ms: 30_000,
                recovery_speech_window_input: None,
                base_worker_budget: Some(1),
                recovery_worker_budget: Some(1),
            },
            request_options.clone(),
        )
        .await?;

    let execution = client
        .execute_recovery_split_with_options(
            AudioShardRecoveryWorkflowRequest {
                parent_plan: &parent_plan,
                materialization: &materialization,
                profile: &profile,
                request_metrics: &[],
                selection_options: no_recovery_selection_options(),
                patch_options: AudioRecoveryPatchGateOptions::default(),
                recovery_split_duration_ms: 30_000,
                recovery_speech_window_input: None,
                base_worker_budget: Some(2),
                recovery_worker_budget: Some(1),
            },
            request_options,
        )
        .await?;

    assert_eq!(execution.base_materialized_shards.len(), 1);
    assert_eq!(
        execution.base_materialized_shards[0].manifest.start_ms,
        60_000
    );
    assert_eq!(execution.base_inputs, parent_inputs);
    assert_eq!(
        execution.base_response.results,
        vec![seed_result.clone(), miss_result.clone()]
    );
    assert_eq!(execution.transcript_admission_stats().planned_hit_count, 1);
    assert_eq!(execution.transcript_admission_stats().planned_miss_count, 1);
    assert_eq!(execution.transcript_admission_stats().miss_count, 1);
    assert_eq!(execution.transcript_admission_stats().stored_count, 1);
    assert_eq!(
        execution.merge_report.text,
        "first admitted text\nsecond fresh text"
    );
    let observed_requests = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?
        .clone();
    assert_eq!(observed_requests.len(), 2);
    assert_eq!(observed_requests[0].row_count, 1);
    assert_eq!(observed_requests[1].row_count, 1);
    assert_eq!(observed_requests[1].windows[0].start_ms, 60_000);

    server_handle.abort();
    Ok(())
}

struct AudioShardTestBatch {
    inputs: Vec<AudioShardInput>,
    results: Vec<AudioShardResult>,
    response_batch: EngineRecordBatch,
}

fn two_pass_parent_audio_plan() -> AudioShardPlan {
    AudioShardPlan {
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
        strategy: "full-coverage".to_owned(),
    }
}

fn preflight_parent_audio_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(60_000),
        },
        chunk_duration_ms: 60_000,
        start_offsets_ms: vec![0],
        window_durations_ms: Vec::new(),
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        strategy: "full-coverage".to_owned(),
    }
}

fn partial_preflight_parent_audio_plan() -> AudioShardPlan {
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
        strategy: "full-coverage".to_owned(),
    }
}

fn partial_preflight_seed_audio_plan() -> AudioShardPlan {
    let mut plan = partial_preflight_parent_audio_plan();
    plan.start_offsets_ms = vec![0];
    plan
}

fn no_recovery_selection_options() -> AudioRiskParentSelectionOptions {
    AudioRiskParentSelectionOptions {
        include_boundaries: false,
        max_chars_per_minute: -1.0,
        max_chinese_ratio: -1.0,
        min_latency_ms: u64::MAX,
        min_repeated_ngram_ratio: 2.0,
        ..AudioRiskParentSelectionOptions::default()
    }
}

fn preflight_materialization(
    tempdir: &tempfile::TempDir,
    output_dir_name: &str,
    ffmpeg_name: &str,
) -> Result<AudioShardMaterializationInput, String> {
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join(ffmpeg_name);
    if ffmpeg_name != "missing_ffmpeg" {
        std::fs::write(
            ffmpeg_path.as_path(),
            "#!/bin/sh\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\nprintf cached > \"$last\"\n",
        )
        .map_err(error_to_string)?;
        make_executable(ffmpeg_path.as_path())?;
    }
    Ok(AudioShardMaterializationInput {
        source_path,
        output_dir: tempdir.path().join(output_dir_name),
        ffmpeg_path,
        artifact_cache_dir: None,
        force: true,
    })
}

fn two_pass_materialization(
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

fn two_pass_base_batch(
    parent_plan: &AudioShardPlan,
    materialization: &AudioShardMaterializationInput,
    profile: &AudioShardWorkerProfile,
) -> Result<AudioShardTestBatch, String> {
    let materialized = materialize_audio_shards(parent_plan, materialization)?;
    let inputs = build_audio_shard_inputs(materialized.as_slice(), profile);
    let results = vec![
        AudioShardResult::succeeded(&inputs[0], "开场介绍", 0.90),
        AudioShardResult::succeeded(&inputs[1], "重复重复重复重复重复重复通用测试会议", 0.80),
        AudioShardResult::succeeded(&inputs[2], "结束总结", 0.90),
    ];
    let response_batch = build_audio_shard_result_batch(results.as_slice())?;

    Ok(AudioShardTestBatch {
        inputs,
        results,
        response_batch,
    })
}

fn two_pass_recovery_batch(
    recovery_plan: &AudioShardPlan,
    materialization: &AudioShardMaterializationInput,
    profile: &AudioShardWorkerProfile,
) -> Result<AudioShardTestBatch, String> {
    let materialized = materialize_audio_shards(recovery_plan, materialization)?;
    let inputs = build_audio_shard_inputs(materialized.as_slice(), profile);
    let results = vec![
        AudioShardResult::succeeded(&inputs[0], "通用会议讨论流程", 0.92),
        AudioShardResult::succeeded(&inputs[1], "主持人介绍测试案例", 0.92),
    ];
    let response_batch = build_audio_shard_result_batch(results.as_slice())?;

    Ok(AudioShardTestBatch {
        inputs,
        results,
        response_batch,
    })
}

fn assert_two_pass_recovery_execution(
    execution: &AudioShardRecoveryWorkflowExecution,
    base_results: &[AudioShardResult],
    recovery_results: &[AudioShardResult],
) -> Result<(), String> {
    assert_eq!(execution.base_inputs.len(), 3);
    assert_eq!(execution.base_response.results, base_results);
    assert_eq!(execution.recovery_inputs.len(), 2);
    let Some(recovery_response) = execution.recovery_response.as_ref() else {
        return Err("expected recovery response".to_owned());
    };
    assert_eq!(recovery_response.results, recovery_results);
    assert_eq!(execution.patch_gate_report.accepted_count, 1);
    assert_eq!(
        execution.merge_report.text,
        "开场介绍\n通用会议讨论流程\n主持人介绍测试案例\n结束总结"
    );
    assert!(execution.merge_report.has_complete_success_coverage());
    Ok(())
}

fn assert_two_pass_trace(execution: &AudioShardRecoveryWorkflowExecution) -> Result<(), String> {
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
            "audio.recovery.materialize_shards",
            "audio.recovery.build_arrow_rows",
            "audio.recovery.call_analyzer_flight",
            "audio.recovery.merge_precision_gate",
        ]
    );
    assert_eq!(
        execution.trace.stages[1]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["audio.base.arrow.input_batch.v1"]
    );
    assert_eq!(
        execution.trace.stages[2]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["audio.base.arrow.result_batch.v1"]
    );
    assert_eq!(
        execution.trace.stages[5]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["audio.recovery.arrow.input_batch.v1"]
    );
    assert_eq!(
        execution.trace.stages[6]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["audio.recovery.arrow.result_batch.v1"]
    );
    let base_input_batch = execution
        .memory_checkpoints
        .get::<EngineRecordBatch>(&WorkflowCheckpointId::new(
            "audio.base.arrow.input_batch.v1",
        ))
        .map_err(error_to_string)?;
    assert_eq!(base_input_batch.num_rows(), 3);
    let recovery_result_batch = execution
        .memory_checkpoints
        .get::<EngineRecordBatch>(&WorkflowCheckpointId::new(
            "audio.recovery.arrow.result_batch.v1",
        ))
        .map_err(error_to_string)?;
    assert_eq!(recovery_result_batch.num_rows(), 2);
    Ok(())
}

fn assert_two_pass_observed_requests(
    observed_requests: &Arc<Mutex<Vec<ObservedAudioShardRequest>>>,
) -> Result<(), String> {
    let observed_requests = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?
        .clone();
    assert_eq!(observed_requests.len(), 2);
    assert_eq!(observed_requests[0].row_count, 3);
    assert_eq!(
        observed_requests[0].worker_budget_header.as_deref(),
        Some("2")
    );
    assert_eq!(observed_requests[1].row_count, 2);
    assert_eq!(
        observed_requests[1].worker_budget_header.as_deref(),
        Some("1")
    );
    assert_eq!(
        observed_requests[1].windows,
        vec![
            ObservedAudioShardWindow {
                start_ms: 60_000,
                duration_ms: 30_000,
                media_start_ms: 60_000,
                media_duration_ms: 30_000,
                reading_order_key: "000000.000000060000".to_owned(),
            },
            ObservedAudioShardWindow {
                start_ms: 90_000,
                duration_ms: 30_000,
                media_start_ms: 90_000,
                media_duration_ms: 30_000,
                reading_order_key: "000001.000000090000".to_owned(),
            },
        ]
    );
    Ok(())
}
