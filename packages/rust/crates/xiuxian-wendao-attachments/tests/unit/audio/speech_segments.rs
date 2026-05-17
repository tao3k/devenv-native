use super::{
    AudioSpeechSegment, build_audio_speech_window_plan, parse_audio_speech_segments_sidecar,
    speech_window_planner_input,
};

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
