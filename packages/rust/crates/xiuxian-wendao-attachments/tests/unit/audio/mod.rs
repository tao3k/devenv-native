//! Unit tests for model-agnostic audio shard contracts.

use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchCandidate, AudioRecoveryPatchDecisionKind, AudioRecoveryPatchGateOptions,
    AudioResultCacheInput, AudioShardInput, AudioShardMaterializationInput, AudioShardPlan,
    AudioShardPlannerInput, AudioShardRequestMetric, AudioShardResult, AudioShardStrategy,
    AudioSourceIdentity, AudioSpeechSegment, AudioSpeechWindowPlannerInput,
    DEFAULT_AUDIO_SHARD_PROFILE, audio_result_cache_key, build_audio_recovery_patch_candidates,
    build_audio_recovery_speech_window_plan_for_inputs, build_audio_recovery_split_plan,
    build_audio_recovery_split_plan_for_inputs, build_audio_shard_plan,
    build_audio_speech_window_plan, gate_audio_recovery_patches, materialize_audio_shards,
    merge_audio_shard_results, merge_audio_shard_results_with_recovery_patches,
    parse_audio_speech_segments_sidecar, plan_audio_shards, select_audio_risk_parent_shards,
};

#[cfg(feature = "audio-shard-arrow")]
use xiuxian_wendao_attachments::audio::{
    AudioShardMaterializedItem, AudioShardWorkerProfile, build_audio_shard_input_batch,
    build_audio_shard_inputs, build_audio_shard_result_batch, decode_audio_shard_result_batches,
};

fn sample_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: "recordings/forum.mp3".to_owned(),
            source_sha256: "a".repeat(64),
            duration_ms: Some(90_000),
        },
        chunk_duration_ms: 30_000,
        start_offsets_ms: vec![0, 30_000, 60_000],
        window_durations_ms: Vec::new(),
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "WAV".to_owned(),
        strategy: "uniform".to_owned(),
    }
}

fn speech_window_planner_input() -> AudioSpeechWindowPlannerInput {
    AudioSpeechWindowPlannerInput {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: "recordings/forum.mp3".to_owned(),
            source_sha256: "a".repeat(64),
            duration_ms: Some(300_000),
        },
        chunk_duration_ms: 30_000,
        limit_chunks: 16,
        speech_segments: vec![
            AudioSpeechSegment {
                index: 0,
                start_ms: 0,
                duration_ms: 4_000,
            },
            AudioSpeechSegment {
                index: 1,
                start_ms: 9_000,
                duration_ms: 3_000,
            },
            AudioSpeechSegment {
                index: 2,
                start_ms: 14_000,
                duration_ms: 3_000,
            },
            AudioSpeechSegment {
                index: 3,
                start_ms: 50_000,
                duration_ms: 45_000,
            },
        ],
        merge_gap_ms: 1_000,
        min_window_ms: 8_000,
        short_merge_gap_ms: Some(3_000),
        max_window_ms: Some(30_000),
        context_before_ms: 500,
        context_after_ms: 700,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
    }
}

fn planner_input() -> AudioShardPlannerInput {
    AudioShardPlannerInput {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: "recordings/forum.mp3".to_owned(),
            source_sha256: "a".repeat(64),
            duration_ms: Some(300_000),
        },
        chunk_duration_ms: 30_000,
        limit_chunks: 3,
        start_offset_ms: 10_000,
        strategy: AudioShardStrategy::Uniform,
        context_before_ms: 2_000,
        context_after_ms: 3_000,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
    }
}

#[test]
fn audio_shard_planner_builds_uniform_offsets_in_rust() -> Result<(), String> {
    let plan = build_audio_shard_plan(&planner_input())?;

    assert_eq!(plan.start_offsets_ms, vec![10_000, 140_000, 270_000]);
    assert_eq!(plan.strategy, "uniform");
    assert_eq!(plan.context_before_ms, 2_000);
    assert_eq!(plan.context_after_ms, 3_000);
    Ok(())
}

#[test]
fn audio_shard_planner_builds_head_offsets_in_rust() -> Result<(), String> {
    let mut input = planner_input();
    input.strategy = AudioShardStrategy::Head;

    let plan = build_audio_shard_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![10_000, 40_000, 70_000]);
    assert_eq!(plan.strategy, "head");
    Ok(())
}

#[test]
fn audio_speech_window_planner_reproduces_short_gap_packing() -> Result<(), String> {
    let plan = build_audio_speech_window_plan(&speech_window_planner_input())?;

    assert_eq!(plan.strategy, "speech-segments");
    assert_eq!(plan.start_offsets_ms, vec![0, 9_000, 50_000, 80_000]);
    assert_eq!(plan.window_durations_ms, vec![4_000, 8_000, 30_000, 15_000]);
    let items = plan_audio_shards(&plan)?;
    assert_eq!(items[0].duration_ms, 4_000);
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].media_duration_ms, 4_700);
    assert_eq!(items[1].start_ms, 9_000);
    assert_eq!(items[1].duration_ms, 8_000);
    assert_eq!(items[1].media_start_ms, 8_500);
    assert_eq!(items[1].media_duration_ms, 9_200);
    assert_ne!(items[0].shard_id, items[1].shard_id);
    Ok(())
}

#[test]
fn audio_speech_window_planner_legacy_short_gap_matches_min_window() -> Result<(), String> {
    let mut input = speech_window_planner_input();
    input.short_merge_gap_ms = None;

    let plan = build_audio_speech_window_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![0, 50_000, 80_000]);
    assert_eq!(plan.window_durations_ms, vec![17_000, 30_000, 15_000]);
    Ok(())
}

#[test]
fn audio_speech_window_planner_sorts_segments_like_python() -> Result<(), String> {
    let mut input = speech_window_planner_input();
    input.speech_segments = vec![
        AudioSpeechSegment {
            index: 2,
            start_ms: 14_000,
            duration_ms: 3_000,
        },
        AudioSpeechSegment {
            index: 0,
            start_ms: 0,
            duration_ms: 4_000,
        },
        AudioSpeechSegment {
            index: 1,
            start_ms: 9_000,
            duration_ms: 3_000,
        },
    ];

    let plan = build_audio_speech_window_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![0, 9_000]);
    assert_eq!(plan.window_durations_ms, vec![4_000, 8_000]);
    Ok(())
}

#[test]
fn audio_speech_window_planner_preserves_uncapped_python_semantics() -> Result<(), String> {
    let mut input = speech_window_planner_input();
    input.limit_chunks = 4;
    input.min_window_ms = 0;
    input.max_window_ms = None;
    input.speech_segments = vec![AudioSpeechSegment {
        index: 0,
        start_ms: 0,
        duration_ms: 75_000,
    }];

    let plan = build_audio_speech_window_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![0]);
    assert_eq!(plan.window_durations_ms, vec![75_000]);
    let items = plan_audio_shards(&plan)?;
    assert_eq!(items[0].duration_ms, 75_000);
    assert_eq!(items[0].media_duration_ms, 75_700);
    Ok(())
}

#[test]
fn audio_speech_window_plan_rejects_invalid_duration_counts() -> Result<(), String> {
    let mut plan = sample_plan();
    plan.window_durations_ms = vec![30_000];

    let Err(error) = plan_audio_shards(&plan) else {
        return Err("mismatched window durations unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("duration count"));
    Ok(())
}

#[test]
fn audio_recovery_split_plan_builds_short_windows_from_parent_plan() -> Result<(), String> {
    let parent_plan = sample_plan();

    let recovery_plan = build_audio_recovery_split_plan(&parent_plan, &[2, 0], 15_000)?;

    assert_eq!(recovery_plan.strategy, "risk-recovery-split");
    assert_eq!(recovery_plan.chunk_duration_ms, 15_000);
    assert_eq!(
        recovery_plan.start_offsets_ms,
        vec![0, 15_000, 60_000, 75_000]
    );
    assert_eq!(
        recovery_plan.window_durations_ms,
        vec![15_000, 15_000, 15_000, 15_000]
    );
    let items = plan_audio_shards(&recovery_plan)?;
    assert_eq!(items.len(), 4);
    assert_eq!(items[0].duration_ms, 15_000);
    assert_ne!(items[0].shard_id, items[1].shard_id);
    Ok(())
}

#[test]
fn audio_recovery_split_plan_preserves_tail_duration() -> Result<(), String> {
    let mut parent_plan = sample_plan();
    parent_plan.window_durations_ms = vec![30_000, 25_000, 30_000];

    let recovery_plan = build_audio_recovery_split_plan(&parent_plan, &[1], 15_000)?;

    assert_eq!(recovery_plan.start_offsets_ms, vec![30_000, 45_000]);
    assert_eq!(recovery_plan.window_durations_ms, vec![15_000, 10_000]);
    Ok(())
}

#[test]
fn audio_recovery_split_plan_accepts_selected_parent_inputs() -> Result<(), String> {
    let parent_plan = sample_plan();
    let mut selected = sample_audio_input("selected", "000002.000000060000");
    selected.start_ms = 60_000;
    selected.duration_ms = 30_000;

    let recovery_plan =
        build_audio_recovery_split_plan_for_inputs(&parent_plan, &[selected], 15_000)?;

    assert_eq!(recovery_plan.start_offsets_ms, vec![60_000, 75_000]);
    assert_eq!(recovery_plan.window_durations_ms, vec![15_000, 15_000]);
    Ok(())
}

#[test]
fn audio_recovery_speech_window_plan_clips_to_failed_parent_windows() -> Result<(), String> {
    let mut parent_plan = sample_plan();
    parent_plan.chunk_duration_ms = 60_000;
    parent_plan.start_offsets_ms = vec![0, 60_000, 120_000];
    parent_plan.source.duration_ms = Some(180_000);
    let mut failed_parent = sample_audio_input("failed", "000001.000000060000");
    failed_parent.start_ms = 60_000;
    failed_parent.duration_ms = 60_000;
    let mut speech_input = speech_window_planner_input();
    speech_input.source = parent_plan.source.clone();
    speech_input.chunk_duration_ms = 15_000;
    speech_input.min_window_ms = 0;
    speech_input.merge_gap_ms = 500;
    speech_input.short_merge_gap_ms = Some(500);
    speech_input.max_window_ms = Some(15_000);
    speech_input.speech_segments = vec![
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
    ];

    let recovery_plan = build_audio_recovery_speech_window_plan_for_inputs(
        &parent_plan,
        &[failed_parent],
        &speech_input,
    )?
    .ok_or_else(|| "speech recovery plan was not produced".to_owned())?;

    assert_eq!(recovery_plan.strategy, "speech-segments");
    assert_eq!(
        recovery_plan.start_offsets_ms,
        vec![60_000, 90_000, 118_000]
    );
    assert_eq!(recovery_plan.window_durations_ms, vec![5_000, 4_000, 2_000]);
    assert_eq!(
        recovery_plan.context_before_ms,
        parent_plan.context_before_ms
    );
    assert_eq!(recovery_plan.sample_rate_hz, parent_plan.sample_rate_hz);
    Ok(())
}

#[test]
fn audio_recovery_speech_window_plan_does_not_merge_across_parent_boundaries() -> Result<(), String>
{
    let mut parent_plan = sample_plan();
    parent_plan.chunk_duration_ms = 60_000;
    parent_plan.start_offsets_ms = vec![0, 60_000, 120_000];
    parent_plan.source.duration_ms = Some(180_000);
    let mut first_failed_parent = sample_audio_input("failed-a", "000000.000000000000");
    first_failed_parent.start_ms = 0;
    first_failed_parent.duration_ms = 60_000;
    let mut second_failed_parent = sample_audio_input("failed-b", "000001.000000060000");
    second_failed_parent.start_ms = 60_000;
    second_failed_parent.duration_ms = 60_000;
    let mut speech_input = speech_window_planner_input();
    speech_input.source = parent_plan.source.clone();
    speech_input.chunk_duration_ms = 15_000;
    speech_input.min_window_ms = 0;
    speech_input.merge_gap_ms = 500;
    speech_input.short_merge_gap_ms = Some(500);
    speech_input.max_window_ms = Some(15_000);
    speech_input.speech_segments = vec![
        AudioSpeechSegment {
            index: 0,
            start_ms: 59_800,
            duration_ms: 200,
        },
        AudioSpeechSegment {
            index: 1,
            start_ms: 60_000,
            duration_ms: 300,
        },
    ];

    let recovery_plan = build_audio_recovery_speech_window_plan_for_inputs(
        &parent_plan,
        &[first_failed_parent, second_failed_parent],
        &speech_input,
    )?
    .ok_or_else(|| "speech recovery plan was not produced".to_owned())?;

    assert_eq!(recovery_plan.start_offsets_ms, vec![59_800, 60_000]);
    assert_eq!(recovery_plan.window_durations_ms, vec![200, 300]);
    let items = plan_audio_shards(&recovery_plan)?;
    assert_eq!(
        items[0].start_ms.saturating_add(items[0].duration_ms),
        60_000
    );
    assert_eq!(items[1].start_ms, 60_000);
    Ok(())
}

#[test]
fn audio_recovery_speech_window_plan_returns_none_when_no_speech_hits_parent() -> Result<(), String>
{
    let parent_plan = sample_plan();
    let mut failed_parent = sample_audio_input("failed", "000001.000000030000");
    failed_parent.start_ms = 30_000;
    failed_parent.duration_ms = 30_000;
    let mut speech_input = speech_window_planner_input();
    speech_input.source = parent_plan.source.clone();
    speech_input.speech_segments = vec![AudioSpeechSegment {
        index: 0,
        start_ms: 70_000,
        duration_ms: 5_000,
    }];

    let recovery_plan = build_audio_recovery_speech_window_plan_for_inputs(
        &parent_plan,
        &[failed_parent],
        &speech_input,
    )?;

    assert!(recovery_plan.is_none());
    Ok(())
}

#[test]
fn audio_recovery_split_plan_rejects_duplicate_or_invalid_parent() -> Result<(), String> {
    let parent_plan = sample_plan();

    let Err(duplicate_error) = build_audio_recovery_split_plan(&parent_plan, &[1, 1], 15_000)
    else {
        return Err("duplicate recovery parent unexpectedly succeeded".to_owned());
    };
    assert!(duplicate_error.contains("duplicated"));

    let Err(range_error) = build_audio_recovery_split_plan(&parent_plan, &[3], 15_000) else {
        return Err("out-of-range recovery parent unexpectedly succeeded".to_owned());
    };
    assert!(range_error.contains("out of range"));
    Ok(())
}

#[test]
fn audio_risk_parent_selection_uses_rust_text_and_latency_facts() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.duration_ms = 60_000;
    let mut second = sample_audio_input("second", "000001.000000060000");
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    let mut third = sample_audio_input("third", "000002.000000120000");
    third.start_ms = 120_000;
    third.duration_ms = 60_000;
    let inputs = vec![second.clone(), third.clone(), first.clone()];
    let results = vec![
        AudioShardResult::succeeded(&first, "开场介绍", 0.9),
        AudioShardResult::succeeded(&second, "重复重复重复重复重复重复家装行业论坛", 0.9),
        AudioShardResult::succeeded(&third, "结束总结", 0.9),
    ];
    let request_metrics = vec![AudioShardRequestMetric {
        shard_element_id: "second".to_owned(),
        wall_ms: 60_000,
    }];

    let selected = select_audio_risk_parent_shards(
        inputs.as_slice(),
        results.as_slice(),
        request_metrics.as_slice(),
        Default::default(),
    )?;

    assert_eq!(
        selected
            .iter()
            .map(|row| row.shard_element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second", "third"]
    );
    assert_eq!(
        selected[0].reasons,
        vec!["low-text-density", "timeline-boundary"]
    );
    assert!(selected[1].reasons.contains(&"high-repetition".to_owned()));
    assert!(selected[1].reasons.contains(&"high-latency".to_owned()));
    assert_eq!(
        selected[2].reasons,
        vec!["low-text-density", "timeline-boundary"]
    );
    Ok(())
}

#[test]
fn audio_risk_parent_selection_reserves_boundaries_under_limit() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.duration_ms = 60_000;
    let mut second = sample_audio_input("second", "000001.000000060000");
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    let mut third = sample_audio_input("third", "000002.000000120000");
    third.start_ms = 120_000;
    third.duration_ms = 60_000;
    let inputs = vec![first.clone(), second.clone(), third.clone()];
    let results = vec![
        AudioShardResult::succeeded(&first, "开场介绍", 0.9),
        AudioShardResult::succeeded(&second, "重复重复重复重复重复重复家装行业论坛", 0.9),
        AudioShardResult::succeeded(&third, "结束总结", 0.9),
    ];
    let mut options = xiuxian_wendao_attachments::audio::AudioRiskParentSelectionOptions {
        limit_parents: 2,
        ..Default::default()
    };

    let selected = select_audio_risk_parent_shards(&inputs, &results, &[], options)?;

    assert_eq!(
        selected
            .iter()
            .map(|row| row.shard_element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "third"]
    );

    options.include_boundaries = false;
    options.max_chars_per_minute = 0.0;
    options.max_chinese_ratio = 0.0;
    let selected_without_boundaries =
        select_audio_risk_parent_shards(&inputs, &results, &[], options)?;
    assert_eq!(
        selected_without_boundaries
            .iter()
            .map(|row| row.shard_element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["second"]
    );
    Ok(())
}

#[test]
fn audio_risk_parent_selection_includes_failed_rows_for_recovery() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.duration_ms = 60_000;
    let mut second = sample_audio_input("second", "000001.000000060000");
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    let inputs = vec![first.clone(), second.clone()];
    let results = vec![
        AudioShardResult::failed(&first, "audio transcript quality gate failed"),
        AudioShardResult::skipped(&second, "not configured"),
    ];
    let options = xiuxian_wendao_attachments::audio::AudioRiskParentSelectionOptions {
        include_boundaries: false,
        ..Default::default()
    };

    let selected = select_audio_risk_parent_shards(&inputs, &results, &[], options)?;

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].shard_element_id, "first");
    assert_eq!(selected[0].reasons, vec!["failed-result"]);
    Ok(())
}

#[test]
fn audio_speech_segments_sidecar_accepts_jsonl_seconds_and_millis() -> Result<(), String> {
    let segments = parse_audio_speech_segments_sidecar(
        r#"
{"startSeconds":2.5,"endSeconds":4.0}
{"startMs":500,"durationMs":250}
"#,
    )?;

    assert_eq!(
        segments,
        vec![
            AudioSpeechSegment {
                index: 0,
                start_ms: 500,
                duration_ms: 250,
            },
            AudioSpeechSegment {
                index: 1,
                start_ms: 2500,
                duration_ms: 1500,
            },
        ]
    );
    Ok(())
}

#[test]
fn audio_speech_segments_sidecar_rejects_invalid_ranges() -> Result<(), String> {
    let Err(error) = parse_audio_speech_segments_sidecar(r#"[{"startMs":4000,"endMs":3000}]"#)
    else {
        return Err("invalid speech segment range unexpectedly parsed".to_owned());
    };

    assert!(error.contains("before start"));
    Ok(())
}

#[test]
fn audio_speech_window_planner_rejects_invalid_limits() -> Result<(), String> {
    let mut input = speech_window_planner_input();
    input.min_window_ms = 31_000;

    let Err(error) = build_audio_speech_window_plan(&input) else {
        return Err("invalid speech window limits unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("min window"));
    Ok(())
}

#[test]
fn audio_shard_manifest_is_backend_independent_and_ordered() -> Result<(), String> {
    let items = plan_audio_shards(&sample_plan())?;

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].source_id, "recordings/forum.mp3");
    assert_eq!(items[0].audio_format, "wav");
    assert_eq!(items[0].reading_order_key, "000000.000000000000");
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].media_duration_ms, 30_000);
    assert_eq!(items[1].reading_order_key, "000001.000000030000");
    assert_ne!(items[0].shard_id, items[1].shard_id);
    assert!(items[0].cache_key.starts_with(DEFAULT_AUDIO_SHARD_PROFILE));
    Ok(())
}

#[test]
fn audio_shard_identity_changes_with_precision_affecting_parameters() -> Result<(), String> {
    let plan = sample_plan();
    let baseline = plan_audio_shards(&plan)?;
    let mut changed = plan;
    changed.sample_rate_hz = 8_000;

    let changed_items = plan_audio_shards(&changed)?;

    assert_ne!(baseline[0].shard_id, changed_items[0].shard_id);
    Ok(())
}

#[test]
fn audio_shard_media_window_preserves_logical_order_with_context() -> Result<(), String> {
    let mut plan = sample_plan();
    plan.context_before_ms = 2_000;
    plan.context_after_ms = 3_000;

    let items = plan_audio_shards(&plan)?;

    assert_eq!(items[0].start_ms, 0);
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].context_before_ms, 0);
    assert_eq!(items[0].context_after_ms, 3_000);
    assert_eq!(items[1].start_ms, 30_000);
    assert_eq!(items[1].media_start_ms, 28_000);
    assert_eq!(items[1].media_duration_ms, 35_000);
    assert_eq!(items[1].reading_order_key, "000001.000000030000");
    Ok(())
}

#[test]
fn audio_shard_identity_changes_with_context() -> Result<(), String> {
    let plan = sample_plan();
    let baseline = plan_audio_shards(&plan)?;
    let mut changed = plan;
    changed.context_after_ms = 1_000;

    let changed_items = plan_audio_shards(&changed)?;

    assert_ne!(baseline[0].shard_id, changed_items[0].shard_id);
    Ok(())
}

#[test]
fn audio_shard_plan_rejects_invalid_contract_inputs() -> Result<(), String> {
    let mut plan = sample_plan();
    plan.chunk_duration_ms = 0;

    let Err(error) = plan_audio_shards(&plan) else {
        return Err("invalid audio shard plan unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("chunk duration"));
    Ok(())
}

#[test]
fn audio_result_cache_key_includes_backend_and_task_identity() -> Result<(), String> {
    let input = AudioResultCacheInput {
        shard_cache_key: "audio-shards-v1:abc".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_id: "hosted-audio".to_owned(),
        backend_config_hash: "model-a".to_owned(),
    };
    let baseline = audio_result_cache_key(&input)?;
    let mut changed = input;
    changed.backend_config_hash = "model-b".to_owned();

    let changed_key = audio_result_cache_key(&changed)?;

    assert!(baseline.starts_with("transcription:hosted-audio:"));
    assert_ne!(baseline, changed_key);
    Ok(())
}

#[test]
fn audio_result_cache_key_rejects_empty_backend_identity() -> Result<(), String> {
    let input = AudioResultCacheInput {
        shard_cache_key: "audio-shards-v1:abc".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_id: String::new(),
        backend_config_hash: "model-a".to_owned(),
    };

    let Err(error) = audio_result_cache_key(&input) else {
        return Err("invalid backend identity unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("backend id"));
    Ok(())
}

#[test]
fn audio_shard_result_merge_preserves_reading_order_and_dedupes_boundary() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000001.000000030000");
    let mut second = sample_audio_input("second", "000000.000000000000");
    first.start_ms = 30_000;
    second.start_ms = 0;
    let inputs = vec![first.clone(), second.clone()];
    let first_result = AudioShardResult::succeeded(&first, "论坛开始，今天讨论行业趋势", 0.9);
    let second_result = AudioShardResult::succeeded(&second, "大家好，论坛开始", 0.9);

    let report = merge_audio_shard_results(&inputs, &[first_result, second_result])?;

    assert_eq!(report.text, "大家好，论坛开始，今天讨论行业趋势");
    assert_eq!(
        report.timeline_text,
        "[00:00.000-00:30.000] 大家好，论坛开始\n[00:30.000-01:00.000] ，今天讨论行业趋势"
    );
    assert_eq!(report.succeeded_count, 2);
    assert!(report.has_complete_success_coverage());
    Ok(())
}

#[test]
fn audio_shard_result_merge_reports_failed_skipped_missing_and_duplicate_rows() -> Result<(), String>
{
    let first = sample_audio_input("first", "000000.000000000000");
    let second = sample_audio_input("second", "000001.000000030000");
    let third = sample_audio_input("third", "000002.000000060000");
    let duplicate = AudioShardResult::succeeded(&first, "duplicate", 0.9);
    let results = vec![
        AudioShardResult::failed(&first, "model failed"),
        duplicate.clone(),
        duplicate,
        AudioShardResult::skipped(&second, "not configured"),
    ];

    let report = merge_audio_shard_results(&[first, second, third], &results)?;

    assert!(!report.has_complete_success_coverage());
    assert_eq!(report.failed_shard_element_ids, vec!["first"]);
    assert_eq!(report.skipped_shard_element_ids, vec!["second"]);
    assert_eq!(report.missing_shard_element_ids, vec!["third"]);
    assert_eq!(report.duplicate_shard_element_ids, vec!["first"]);
    Ok(())
}

#[test]
fn audio_shard_result_merge_rejects_hash_mismatch() -> Result<(), String> {
    let input = sample_audio_input("first", "000000.000000000000");
    let mut result = AudioShardResult::succeeded(&input, "text", 0.9);
    result.shard_sha256 = "different".to_owned();

    let Err(error) = merge_audio_shard_results(&[input], &[result]) else {
        return Err("hash mismatch should be rejected".to_owned());
    };

    assert!(error.contains("shard hash mismatch"));
    Ok(())
}

#[test]
fn audio_recovery_patch_gate_accepts_short_window_precision_gain() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery_a = sample_audio_input("recovery-a", "000000.000000000000");
    let recovery_b = sample_audio_input("recovery-b", "000001.000000030000");
    let base_result = AudioShardResult::succeeded(
        &parent,
        "装修装修装修装修装修装修装修装修装修装修行业论坛",
        0.8,
    );
    let recovery_results = vec![
        AudioShardResult::succeeded(&recovery_a, "家装行业论坛今天讨论供应链", 0.9),
        AudioShardResult::succeeded(&recovery_b, "主持人介绍长春市场案例", 0.9),
    ];
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery-a".to_owned(), "recovery-b".to_owned()],
    }];

    let (merge_report, gate_report) = merge_audio_shard_results_with_recovery_patches(
        &[parent],
        &[base_result],
        recovery_results.as_slice(),
        candidates.as_slice(),
        AudioRecoveryPatchGateOptions::default(),
    )?;

    assert_eq!(gate_report.accepted_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::AcceptPatch
    );
    assert_eq!(
        merge_report.text,
        "家装行业论坛今天讨论供应链\n主持人介绍长春市场案例"
    );
    assert!(merge_report.has_complete_success_coverage());
    Ok(())
}

#[test]
fn audio_recovery_patch_gate_rejects_precision_regression() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery = sample_audio_input("recovery", "000000.000000000000");
    let base_result = AudioShardResult::succeeded(&parent, "家装行业论坛讨论供应链案例", 0.8);
    let recovery_result = AudioShardResult::succeeded(&recovery, "aaaaaa", 0.9);
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery".to_owned()],
    }];

    let gate_report = gate_audio_recovery_patches(
        &[base_result],
        &[recovery_result],
        candidates.as_slice(),
        AudioRecoveryPatchGateOptions::default(),
    )?;

    assert_eq!(gate_report.accepted_count, 0);
    assert_eq!(gate_report.rejected_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::RejectPatch
    );
    assert!(
        gate_report.decisions[0]
            .rejection_reasons
            .contains(&"chinese-ratio-drop".to_owned())
    );
    assert!(
        gate_report.decisions[0]
            .rejection_reasons
            .contains(&"char-collapse".to_owned())
    );
    Ok(())
}

#[test]
fn audio_recovery_patch_gate_accepts_recovery_for_failed_parent() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery = sample_audio_input("recovery", "000000.000000000000");
    let base_result = AudioShardResult::failed(&parent, "audio transcript quality gate failed");
    let recovery_result = AudioShardResult::succeeded(&recovery, "今天讨论家居行业供应链", 0.9);
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery".to_owned()],
    }];

    let (merge_report, gate_report) = merge_audio_shard_results_with_recovery_patches(
        &[parent],
        &[base_result],
        &[recovery_result],
        candidates.as_slice(),
        AudioRecoveryPatchGateOptions::default(),
    )?;

    assert_eq!(gate_report.accepted_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::AcceptPatch
    );
    assert_eq!(merge_report.text, "今天讨论家居行业供应链");
    assert!(merge_report.has_complete_success_coverage());
    Ok(())
}

#[test]
fn audio_recovery_patch_candidates_group_recovery_windows_under_parent() -> Result<(), String> {
    let mut parent_a = sample_audio_input("parent-a", "000000.000000000000");
    parent_a.start_ms = 0;
    parent_a.duration_ms = 60_000;
    let mut parent_b = sample_audio_input("parent-b", "000001.000000060000");
    parent_b.start_ms = 60_000;
    parent_b.duration_ms = 60_000;
    let mut recovery_b = sample_audio_input("recovery-b", "000003.000000090000");
    recovery_b.start_ms = 90_000;
    recovery_b.duration_ms = 30_000;
    let mut recovery_a = sample_audio_input("recovery-a", "000002.000000030000");
    recovery_a.start_ms = 30_000;
    recovery_a.duration_ms = 30_000;

    let candidates =
        build_audio_recovery_patch_candidates(&[parent_b, parent_a], &[recovery_b, recovery_a])?;

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].parent_shard_element_id, "parent-a");
    assert_eq!(candidates[0].recovery_shard_element_ids, vec!["recovery-a"]);
    assert_eq!(candidates[1].parent_shard_element_id, "parent-b");
    assert_eq!(candidates[1].recovery_shard_element_ids, vec!["recovery-b"]);
    Ok(())
}

#[test]
fn audio_recovery_patch_candidates_reject_unowned_windows() -> Result<(), String> {
    let mut parent = sample_audio_input("parent", "000000.000000000000");
    parent.start_ms = 0;
    parent.duration_ms = 60_000;
    let mut recovery = sample_audio_input("recovery", "000002.000000120000");
    recovery.start_ms = 120_000;
    recovery.duration_ms = 30_000;

    let Err(error) = build_audio_recovery_patch_candidates(&[parent], &[recovery]) else {
        return Err("unowned recovery window unexpectedly mapped".to_owned());
    };

    assert!(error.contains("no parent logical window"));
    Ok(())
}

#[test]
fn audio_materialization_runs_splitter_with_planned_media_windows() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    let log_path = tempdir.path().join("ffmpeg.log");
    std::fs::write(
        ffmpeg_path.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\ntouch \"$last\"\n",
            log_path.display()
        ),
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![30_000];
    plan.context_before_ms = 2_000;
    plan.context_after_ms = 3_000;
    let input = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: tempdir.path().join("chunks"),
        ffmpeg_path,
        force: true,
    };

    let items = materialize_audio_shards(&plan, &input)?;

    assert_eq!(items.len(), 1);
    assert!(items[0].output_path.exists());
    assert_eq!(items[0].manifest.media_start_ms, 28_000);
    assert_eq!(items[0].manifest.media_duration_ms, 35_000);
    let log = std::fs::read_to_string(log_path).map_err(error_to_string)?;
    assert!(log.contains("-ss"));
    assert!(log.contains("28.000"));
    assert!(log.contains("-t"));
    assert!(log.contains("35.000"));
    assert!(log.contains(source_path.to_string_lossy().as_ref()));
    Ok(())
}

#[test]
fn audio_materialization_reuses_existing_chunks_without_splitter() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("missing_ffmpeg");
    let output_dir = tempdir.path().join("chunks");
    std::fs::create_dir_all(output_dir.as_path()).map_err(error_to_string)?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![0];
    let existing_manifest = plan_audio_shards(&plan)?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let existing_path = output_dir.join(format!(
        "audio_{:06}_{}.{}",
        existing_manifest.chunk_index,
        existing_manifest
            .shard_id
            .chars()
            .take(16)
            .collect::<String>(),
        existing_manifest.audio_format
    ));
    std::fs::write(existing_path.as_path(), b"cached").map_err(error_to_string)?;
    let input = AudioShardMaterializationInput {
        source_path,
        output_dir,
        ffmpeg_path,
        force: false,
    };

    let items = materialize_audio_shards(&plan, &input)?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].output_path, existing_path);
    Ok(())
}

#[cfg(feature = "audio-shard-arrow")]
#[test]
fn audio_shard_arrow_contract_roundtrips_results() -> Result<(), String> {
    let manifest = plan_audio_shards(&sample_plan())?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let materialized = AudioShardMaterializedItem {
        manifest,
        output_path: std::path::PathBuf::from("/tmp/audio.wav"),
        shard_sha256: "b".repeat(64),
    };
    let profile = AudioShardWorkerProfile::transcription("hosted-audio");

    let inputs = build_audio_shard_inputs(&[materialized], &profile);
    let input_batch = build_audio_shard_input_batch(inputs.as_slice())?;
    let success = AudioShardResult::succeeded(&inputs[0], "transcript", 0.88);
    let result_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let decoded = decode_audio_shard_result_batches(&[result_batch])?;

    assert_eq!(input_batch.num_rows(), 1);
    assert_eq!(
        inputs[0].contract_version,
        "xiuxian_wendao.audio_shard_input.v1"
    );
    assert_eq!(inputs[0].task_profile, "transcription");
    assert_eq!(inputs[0].backend_profile, "hosted-audio");
    assert_eq!(decoded, vec![success]);
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .map_err(error_to_string)?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).map_err(error_to_string)
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

fn error_to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn sample_audio_input(shard_element_id: &str, reading_order_key: &str) -> AudioShardInput {
    AudioShardInput {
        contract_version: "xiuxian_wendao.audio_shard_input.v1".to_owned(),
        source_path: "/tmp/source.mp3".to_owned(),
        source_content_hash: "sourcehash".to_owned(),
        shard_path: format!("/tmp/{shard_element_id}.wav"),
        shard_sha256: format!("shardhash-{shard_element_id}"),
        shard_profile: "audio-shards-v1".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_profile: "hosted-audio-transcript-v1".to_owned(),
        preferred_languages: vec!["zh".to_owned()],
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        start_ms: 0,
        duration_ms: 30_000,
        media_start_ms: 0,
        media_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        shard_element_id: shard_element_id.to_owned(),
        reading_order_key: reading_order_key.to_owned(),
    }
}
