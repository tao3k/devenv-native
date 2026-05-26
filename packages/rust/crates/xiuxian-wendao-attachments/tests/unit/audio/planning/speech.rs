use crate::audio::{
    AudioSpeechSegment, build_audio_speech_window_plan, plan_audio_shards, sample_plan,
    speech_window_planner_input,
};

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
fn audio_speech_window_planner_snaps_near_cap_segment_boundary() -> Result<(), String> {
    let mut input = speech_window_planner_input();
    input.min_window_ms = 2_000;
    input.max_window_ms = Some(28_000);
    input.boundary_snap_tolerance_ms = 2_000;
    input.speech_segments = vec![
        AudioSpeechSegment {
            index: 0,
            start_ms: 0,
            duration_ms: 29_859,
        },
        AudioSpeechSegment {
            index: 1,
            start_ms: 35_000,
            duration_ms: 3_000,
        },
    ];

    let plan = build_audio_speech_window_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![0, 35_000]);
    assert_eq!(plan.window_durations_ms, vec![29_859, 3_000]);
    Ok(())
}

#[test]
fn audio_speech_window_planner_balances_short_tail_splits() -> Result<(), String> {
    let mut input = speech_window_planner_input();
    input.min_window_ms = 2_000;
    input.max_window_ms = Some(28_000);
    input.boundary_snap_tolerance_ms = 0;
    input.speech_segments = vec![AudioSpeechSegment {
        index: 0,
        start_ms: 0,
        duration_ms: 29_859,
    }];

    let plan = build_audio_speech_window_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![0, 14_930]);
    assert_eq!(plan.window_durations_ms, vec![14_930, 14_929]);
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
fn audio_plan_identity_includes_optional_bitrate() -> Result<(), String> {
    let mut baseline = sample_plan();
    baseline.start_offsets_ms = vec![0];
    let mut compressed = baseline.clone();
    compressed.audio_bitrate = Some("96k".to_owned());

    let baseline_item = plan_audio_shards(&baseline)?
        .into_iter()
        .next()
        .ok_or_else(|| "expected baseline shard".to_owned())?;
    let compressed_item = plan_audio_shards(&compressed)?
        .into_iter()
        .next()
        .ok_or_else(|| "expected compressed shard".to_owned())?;

    assert_eq!(compressed_item.audio_bitrate.as_deref(), Some("96k"));
    assert_ne!(baseline_item.shard_id, compressed_item.shard_id);
    assert_ne!(baseline_item.cache_key, compressed_item.cache_key);
    Ok(())
}
