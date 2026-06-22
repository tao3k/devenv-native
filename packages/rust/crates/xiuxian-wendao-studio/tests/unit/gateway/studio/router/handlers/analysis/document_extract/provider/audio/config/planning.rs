use std::path::Path;

use crate::studio::router::handlers::analysis::document_extract::provider::audio::{
    audio_recovery_selection_options_for_plan, base_speech_window_plan_from_config,
    build_full_coverage_audio_plan, document_extract_audio_config,
    recovery_speech_window_input_from_config,
};

#[test]
fn full_coverage_plan_preserves_tail_window_duration() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS").then(|| "60000".to_owned())
    })?;
    let plan = build_full_coverage_audio_plan(
        Path::new("/tmp/input.mp3"),
        "sourcehash".to_owned(),
        125_500,
        &config,
    )?;

    assert_eq!(plan.start_offsets_ms, vec![0, 60_000, 120_000]);
    assert_eq!(plan.window_durations_ms, vec![60_000, 60_000, 5_500]);
    assert_eq!(plan.strategy, "full-coverage");
    Ok(())
}

#[test]
fn audio_recovery_speech_sidecar_builds_planner_input() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let sidecar_path = temp.path().join("segments.jsonl");
    std::fs::write(
        sidecar_path.as_path(),
        "{\"startSeconds\":1.0,\"durationSeconds\":2.0}\n{\"startMs\":7000,\"endMs\":9000}\n",
    )
    .map_err(|error| error.to_string())?;
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS" => Some("15000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL" => {
            Some(sidecar_path.to_string_lossy().to_string())
        }
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS" => Some("250".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS" => Some("1000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MAX_WINDOW_MS" => Some("12000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_BOUNDARY_SNAP_TOLERANCE_MS" => Some("750".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_BITRATE" => Some("96k".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_LIMIT_CHUNKS" => Some("8".to_owned()),
        _ => None,
    })?;
    let plan = build_full_coverage_audio_plan(
        Path::new("/tmp/input.mp3"),
        "sourcehash".to_owned(),
        30_000,
        &config,
    )?;

    let speech_input = recovery_speech_window_input_from_config(&plan, &config)?
        .ok_or_else(|| "expected speech window input".to_owned())?;

    assert_eq!(speech_input.chunk_duration_ms, 15_000);
    assert_eq!(speech_input.limit_chunks, 8);
    assert_eq!(speech_input.merge_gap_ms, 250);
    assert_eq!(speech_input.min_window_ms, 1000);
    assert_eq!(speech_input.max_window_ms, Some(12_000));
    assert_eq!(speech_input.boundary_snap_tolerance_ms, 750);
    assert_eq!(speech_input.audio_bitrate.as_deref(), Some("96k"));
    assert_eq!(speech_input.speech_segments.len(), 2);
    assert_eq!(speech_input.speech_segments[0].start_ms, 1000);
    assert_eq!(speech_input.speech_segments[1].duration_ms, 2000);
    Ok(())
}

#[test]
fn audio_base_speech_sidecar_builds_speech_window_plan() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let sidecar_path = temp.path().join("segments.jsonl");
    std::fs::write(
        sidecar_path.as_path(),
        "{\"startSeconds\":1.0,\"durationSeconds\":2.0}\n{\"startMs\":7000,\"endMs\":9000}\n",
    )
    .map_err(|error| error.to_string())?;
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS" => Some("30000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL" => {
            Some(sidecar_path.to_string_lossy().to_string())
        }
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS" => Some("250".to_owned()),
        _ => None,
    })?;
    let full_plan = build_full_coverage_audio_plan(
        Path::new("/tmp/input.mp3"),
        "sourcehash".to_owned(),
        30_000,
        &config,
    )?;

    let speech_plan = base_speech_window_plan_from_config(&full_plan, &config)?
        .ok_or_else(|| "expected base speech-window plan".to_owned())?;

    assert_eq!(speech_plan.strategy, "speech-segments");
    assert_eq!(speech_plan.chunk_duration_ms, 30_000);
    assert_eq!(speech_plan.start_offsets_ms, vec![1000, 7000]);
    assert_eq!(speech_plan.window_durations_ms, vec![2000, 2000]);
    assert_eq!(speech_plan.source, full_plan.source);
    Ok(())
}

#[test]
fn audio_base_speech_sidecar_uses_explicit_max_window() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let sidecar_path = temp.path().join("segments.jsonl");
    std::fs::write(
        sidecar_path.as_path(),
        "{\"startMs\":1000,\"durationMs\":45000}\n",
    )
    .map_err(|error| error.to_string())?;
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS" => Some("30000".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL" => {
            Some(sidecar_path.to_string_lossy().to_string())
        }
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MAX_WINDOW_MS" => Some("12000".to_owned()),
        _ => None,
    })?;
    let full_plan = build_full_coverage_audio_plan(
        Path::new("/tmp/input.mp3"),
        "sourcehash".to_owned(),
        60_000,
        &config,
    )?;

    let speech_plan = base_speech_window_plan_from_config(&full_plan, &config)?
        .ok_or_else(|| "expected base speech-window plan".to_owned())?;

    assert_eq!(speech_plan.chunk_duration_ms, 30_000);
    assert_eq!(
        speech_plan.start_offsets_ms,
        vec![1000, 13000, 25000, 37000]
    );
    assert_eq!(
        speech_plan.window_durations_ms,
        vec![12000, 12000, 12000, 9000]
    );
    Ok(())
}

#[test]
fn audio_base_speech_sidecar_merges_short_windows_by_min_window() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let sidecar_path = temp.path().join("segments.jsonl");
    std::fs::write(
        sidecar_path.as_path(),
        "{\"startMs\":1000,\"endMs\":1334}\n{\"startMs\":2000,\"endMs\":5000}\n",
    )
    .map_err(|error| error.to_string())?;
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL" => {
            Some(sidecar_path.to_string_lossy().to_string())
        }
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS" => Some("500".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS" => Some("2000".to_owned()),
        _ => None,
    })?;
    let full_plan = build_full_coverage_audio_plan(
        Path::new("/tmp/input.mp3"),
        "sourcehash".to_owned(),
        30_000,
        &config,
    )?;

    let speech_plan = base_speech_window_plan_from_config(&full_plan, &config)?
        .ok_or_else(|| "expected base speech-window plan".to_owned())?;

    assert_eq!(speech_plan.start_offsets_ms, vec![1000]);
    assert_eq!(speech_plan.window_durations_ms, vec![4000]);
    Ok(())
}

#[test]
fn audio_speech_segment_plan_uses_hard_failure_recovery_only() -> Result<(), String> {
    let config = document_extract_audio_config(&|_| None)?;
    let mut plan = build_full_coverage_audio_plan(
        Path::new("/tmp/input.mp3"),
        "sourcehash".to_owned(),
        30_000,
        &config,
    )?;

    let full_coverage_options = audio_recovery_selection_options_for_plan(&plan);
    assert!(full_coverage_options.include_boundaries);

    plan.strategy = "speech-segments".to_owned();
    let speech_options = audio_recovery_selection_options_for_plan(&plan);
    assert!(!speech_options.include_boundaries);
    assert_eq!(speech_options.max_chars_per_minute, -1.0);
    assert_eq!(speech_options.max_chinese_ratio, -1.0);
    assert_eq!(speech_options.min_latency_ms, u64::MAX);
    assert_eq!(speech_options.min_repeated_ngram_ratio, 2.0);
    Ok(())
}
