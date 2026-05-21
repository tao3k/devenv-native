use std::path::Path;

use crate::studio::router::handlers::analysis::document_extract::provider::audio::{
    audio_worker_budget_with_lookup, build_full_coverage_audio_plan, document_extract_audio_config,
    parse_ffprobe_duration_ms, recovery_speech_window_input_from_config,
};

#[test]
fn audio_config_defaults_are_model_neutral() -> Result<(), String> {
    let config = document_extract_audio_config(&|_| None)?;

    assert_eq!(config.backend_profile, "hosted-audio-transcript-v1");
    assert_eq!(config.chunk_duration_ms, 60_000);
    assert_eq!(config.recovery_split_duration_ms, 30_000);
    assert_eq!(config.base_worker_budget, None);
    assert_eq!(config.recovery_worker_budget, None);
    assert_eq!(config.speech_segments_jsonl_path, None);
    assert_eq!(config.speech_merge_gap_ms, 500);
    assert_eq!(config.speech_min_window_ms, 0);
    assert_eq!(config.speech_limit_chunks, 10_000);
    Ok(())
}

#[test]
fn audio_worker_budget_accepts_auto_or_positive_integer() -> Result<(), String> {
    assert_eq!(
        audio_worker_budget_with_lookup(&|_| Some("auto".to_owned()), "TEST")?,
        None
    );
    assert_eq!(
        audio_worker_budget_with_lookup(&|_| Some("4".to_owned()), "TEST")?,
        Some(4)
    );
    assert!(audio_worker_budget_with_lookup(&|_| Some("0".to_owned()), "TEST").is_err());
    Ok(())
}

#[test]
fn parses_ffprobe_duration_as_ceil_milliseconds() -> Result<(), String> {
    assert_eq!(parse_ffprobe_duration_ms("1.2341")?, 1235);
    assert!(parse_ffprobe_duration_ms("0").is_err());
    assert!(parse_ffprobe_duration_ms("not-a-duration").is_err());
    Ok(())
}

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
    assert_eq!(speech_input.max_window_ms, Some(15_000));
    assert_eq!(speech_input.speech_segments.len(), 2);
    assert_eq!(speech_input.speech_segments[0].start_ms, 1000);
    assert_eq!(speech_input.speech_segments[1].duration_ms, 2000);
    Ok(())
}
