use crate::studio::document_extract_audio_client::{
    AudioShardFlightResponse, AudioShardRecoveryPlanRequest,
};
use crate::unit::gateway::studio::document_extract_audio_client::support::sample_input;
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchDecisionKind, AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions,
    AudioShardPlan, AudioShardRequestMetric, AudioShardResult, AudioSourceIdentity,
    build_audio_recovery_split_plan,
};

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
            "重复重复重复重复重复重复通用会议",
            0.80,
        )],
    };
    let recovery_response = AudioShardFlightResponse {
        results: vec![
            AudioShardResult::succeeded(&first_recovery_input, "通用会议讨论流程", 0.92),
            AudioShardResult::succeeded(&second_recovery_input, "主持人介绍测试案例", 0.92),
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
    assert_eq!(merge_report.text, "通用会议讨论流程\n主持人介绍测试案例");
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
        audio_bitrate: None,
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
            AudioShardResult::succeeded(&second, "重复重复重复重复重复重复通用测试会议", 0.9),
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
    let expected_plan = build_audio_recovery_split_plan(&parent_plan, &[1], 30_000)?;
    assert_eq!(
        planning.recovery_plan.start_offsets_ms,
        expected_plan.start_offsets_ms
    );
    Ok(())
}

#[tokio::test]
async fn audio_shard_response_skips_successful_noop_recovery_split() -> Result<(), String> {
    let parent_plan = noop_recovery_parent_audio_plan();
    let mut first = sample_input();
    first.shard_element_id = "first".to_owned();
    first.start_ms = 0;
    first.duration_ms = 30_000;
    first.reading_order_key = "000000.000000000000".to_owned();
    let mut second = sample_input();
    second.shard_element_id = "second".to_owned();
    second.start_ms = 30_000;
    second.duration_ms = 30_000;
    second.reading_order_key = "000001.000000030000".to_owned();
    let response = AudioShardFlightResponse {
        results: vec![
            AudioShardResult::succeeded(&first, "general opening", 0.9),
            AudioShardResult::succeeded(&second, "general closing", 0.9),
        ],
    };
    let inputs = vec![first, second];

    let planning = response.plan_recovery_split(AudioShardRecoveryPlanRequest {
        parent_plan: &parent_plan,
        inputs: inputs.as_slice(),
        request_metrics: &[],
        selection_options: AudioRiskParentSelectionOptions::default(),
        split_duration_ms: 30_000,
        speech_window_input: None,
    })?;

    assert!(planning.selected_parent_inputs.is_empty());
    assert!(planning.selections.is_empty());
    assert!(planning.recovery_plan.start_offsets_ms.is_empty());
    Ok(())
}

#[tokio::test]
async fn audio_shard_response_keeps_failed_noop_recovery_retry() -> Result<(), String> {
    let parent_plan = noop_recovery_parent_audio_plan();
    let mut first = sample_input();
    first.shard_element_id = "first".to_owned();
    first.start_ms = 0;
    first.duration_ms = 30_000;
    first.reading_order_key = "000000.000000000000".to_owned();
    let response = AudioShardFlightResponse {
        results: vec![AudioShardResult::failed(
            &first,
            "hosted audio worker returned empty text",
        )],
    };

    let planning = response.plan_recovery_split(AudioShardRecoveryPlanRequest {
        parent_plan: &parent_plan,
        inputs: std::slice::from_ref(&first),
        request_metrics: &[],
        selection_options: AudioRiskParentSelectionOptions {
            include_boundaries: false,
            ..Default::default()
        },
        split_duration_ms: 30_000,
        speech_window_input: None,
    })?;

    assert_eq!(planning.selected_parent_inputs.len(), 1);
    assert!(
        planning.selections[0]
            .reasons
            .contains(&"failed-result".to_owned())
    );
    assert_eq!(planning.recovery_plan.start_offsets_ms, vec![0]);
    assert_eq!(planning.recovery_plan.window_durations_ms, vec![30_000]);
    Ok(())
}

fn noop_recovery_parent_audio_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(60_000),
        },
        chunk_duration_ms: 30_000,
        start_offsets_ms: vec![0, 30_000],
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
