use crate::audio::{
    AudioSpeechSegment, build_audio_recovery_speech_window_plan_for_inputs,
    build_audio_recovery_split_plan, build_audio_recovery_split_plan_for_inputs, plan_audio_shards,
    sample_audio_input, sample_plan, speech_window_planner_input,
};

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
