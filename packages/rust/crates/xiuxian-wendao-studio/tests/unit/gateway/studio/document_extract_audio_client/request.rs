use std::sync::{Arc, Mutex};

use arrow::record_batch::RecordBatch as EngineRecordBatch;

use super::support::{
    ObservedAudioShardWindow, error_to_string, make_executable, sample_input,
    spawn_audio_shard_sequence_service, spawn_audio_shard_service,
};
use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightResponse, AudioShardRecoveryPlanRequest,
    AudioShardRecoveryWorkflowRequest,
};
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchDecisionKind, AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions,
    AudioShardMaterializationInput, AudioShardPlan, AudioShardRequestMetric, AudioShardResult,
    AudioShardWorkerProfile, AudioSourceIdentity, build_audio_recovery_split_plan,
    build_audio_shard_inputs, build_audio_shard_result_batch, materialize_audio_shards,
};

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

#[tokio::test]
async fn audio_shard_response_merges_accepted_recovery_patch() -> Result<(), String> {
    let mut base_input = sample_input();
    base_input.duration_ms = 60_000;
    let mut first_recovery_input = sample_input();
    first_recovery_input.shard_element_id = "recovery-a".to_owned();
    first_recovery_input.start_ms = 0;
    first_recovery_input.duration_ms = 30_000;
    first_recovery_input.reading_order_key = "000000.000000000000".to_owned();
    let mut second_recovery_input = sample_input();
    second_recovery_input.shard_element_id = "recovery-b".to_owned();
    second_recovery_input.start_ms = 30_000;
    second_recovery_input.duration_ms = 30_000;
    second_recovery_input.reading_order_key = "000001.000000030000".to_owned();
    let base_response = AudioShardFlightResponse {
        results: vec![AudioShardResult::succeeded(
            &base_input,
            "重复重复重复重复重复重复重复家装论坛",
            0.80,
        )],
    };
    let recovery_response = AudioShardFlightResponse {
        results: vec![
            AudioShardResult::succeeded(&first_recovery_input, "家装论坛讨论供应链", 0.92),
            AudioShardResult::succeeded(&second_recovery_input, "主持人介绍长春案例", 0.92),
        ],
    };

    let (merge_report, gate_report) = base_response.merge_with_recovery_for_inputs(
        std::slice::from_ref(&base_input),
        &[first_recovery_input, second_recovery_input],
        &recovery_response,
        AudioRecoveryPatchGateOptions::default(),
    )?;

    assert_eq!(gate_report.accepted_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::AcceptPatch
    );
    assert_eq!(merge_report.text, "家装论坛讨论供应链\n主持人介绍长春案例");
    assert!(merge_report.has_complete_success_coverage());
    Ok(())
}

#[tokio::test]
async fn audio_shard_response_plans_recovery_split_from_base_quality() -> Result<(), String> {
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
        strategy: "full-coverage".to_owned(),
    };
    let mut first = sample_input();
    first.shard_element_id = "first".to_owned();
    first.duration_ms = 60_000;
    first.reading_order_key = "000000.000000000000".to_owned();
    let mut second = sample_input();
    second.shard_element_id = "second".to_owned();
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    second.reading_order_key = "000001.000000060000".to_owned();
    let mut third = sample_input();
    third.shard_element_id = "third".to_owned();
    third.start_ms = 120_000;
    third.duration_ms = 60_000;
    third.reading_order_key = "000002.000000120000".to_owned();
    let response = AudioShardFlightResponse {
        results: vec![
            AudioShardResult::succeeded(&first, "开场介绍", 0.9),
            AudioShardResult::succeeded(&second, "重复重复重复重复重复重复家装行业论坛", 0.9),
            AudioShardResult::succeeded(&third, "结束总结", 0.9),
        ],
    };
    let options = AudioRiskParentSelectionOptions {
        include_boundaries: false,
        max_chars_per_minute: 0.0,
        max_chinese_ratio: 0.0,
        ..Default::default()
    };
    let metrics = vec![AudioShardRequestMetric {
        shard_element_id: "second".to_owned(),
        wall_ms: 60_000,
    }];

    let inputs = vec![first, second, third];
    let planning = response.plan_recovery_split(AudioShardRecoveryPlanRequest {
        parent_plan: &parent_plan,
        inputs: inputs.as_slice(),
        request_metrics: metrics.as_slice(),
        selection_options: options,
        split_duration_ms: 30_000,
        speech_window_input: None,
    })?;

    assert_eq!(planning.selected_parent_inputs.len(), 1);
    assert_eq!(
        planning.selected_parent_inputs[0].shard_element_id,
        "second"
    );
    assert_eq!(planning.selections[0].shard_element_id, "second");
    assert!(
        planning.selections[0]
            .reasons
            .contains(&"high-repetition".to_owned())
    );
    assert!(
        planning.selections[0]
            .reasons
            .contains(&"high-latency".to_owned())
    );
    assert_eq!(planning.recovery_plan.strategy, "risk-recovery-split");
    assert_eq!(
        planning.recovery_plan.start_offsets_ms,
        vec![60_000, 90_000]
    );
    assert_eq!(
        planning.recovery_plan.window_durations_ms,
        vec![30_000, 30_000]
    );
    Ok(())
}

#[tokio::test]
async fn audio_shard_client_executes_two_pass_recovery_split() -> Result<(), String> {
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
        strategy: "full-coverage".to_owned(),
    };
    let materialization = AudioShardMaterializationInput {
        source_path,
        output_dir: tempdir.path().join("chunks"),
        ffmpeg_path,
        force: true,
    };
    let profile = AudioShardWorkerProfile::transcription("hosted-audio-transcript-v1");
    let base_materialized = materialize_audio_shards(&parent_plan, &materialization)?;
    let base_inputs = build_audio_shard_inputs(base_materialized.as_slice(), &profile);
    let base_results = vec![
        AudioShardResult::succeeded(&base_inputs[0], "开场介绍", 0.90),
        AudioShardResult::succeeded(
            &base_inputs[1],
            "重复重复重复重复重复重复家装行业论坛",
            0.80,
        ),
        AudioShardResult::succeeded(&base_inputs[2], "结束总结", 0.90),
    ];
    let base_batch = build_audio_shard_result_batch(base_results.as_slice())?;

    let recovery_plan = build_audio_recovery_split_plan(&parent_plan, &[1], 30_000)?;
    let recovery_materialized = materialize_audio_shards(&recovery_plan, &materialization)?;
    let recovery_inputs = build_audio_shard_inputs(recovery_materialized.as_slice(), &profile);
    let recovery_results = vec![
        AudioShardResult::succeeded(&recovery_inputs[0], "家装论坛讨论供应链", 0.92),
        AudioShardResult::succeeded(&recovery_inputs[1], "主持人介绍长春案例", 0.92),
    ];
    let recovery_batch = build_audio_shard_result_batch(recovery_results.as_slice())?;
    let cached_materialization = AudioShardMaterializationInput {
        force: false,
        ..materialization
    };
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_audio_shard_sequence_service(
        vec![base_batch, recovery_batch],
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
                shard_element_id: base_inputs[1].shard_element_id.clone(),
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

    assert_eq!(execution.base_inputs.len(), 3);
    assert_eq!(execution.base_response.results, base_results);
    assert_eq!(execution.recovery_inputs.len(), 2);
    assert_eq!(
        execution
            .recovery_response
            .as_ref()
            .expect("recovery response")
            .results,
        recovery_results
    );
    assert_eq!(execution.patch_gate_report.accepted_count, 1);
    assert_eq!(
        execution.merge_report.text,
        "开场介绍\n家装论坛讨论供应链\n主持人介绍长春案例\n结束总结"
    );
    assert!(execution.merge_report.has_complete_success_coverage());
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
        .get::<EngineRecordBatch>("audio.base.arrow.input_batch.v1")
        .map_err(error_to_string)?;
    assert_eq!(base_input_batch.num_rows(), 3);
    let recovery_result_batch = execution
        .memory_checkpoints
        .get::<EngineRecordBatch>("audio.recovery.arrow.result_batch.v1")
        .map_err(error_to_string)?;
    assert_eq!(recovery_result_batch.num_rows(), 2);
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

    server_handle.abort();
    Ok(())
}
