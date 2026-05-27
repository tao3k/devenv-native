use std::path::Path;

use crate::studio::router::handlers::analysis::document_extract::provider::audio::{
    audio_worker_budget_with_lookup, document_extract_audio_config, parse_ffprobe_duration_ms,
};
use xiuxian_llm::model_routing::{DEFAULT_WENDAO_VLLM_SR_BASE_URL, WendaoModelRoutingMode};

#[test]
fn audio_config_defaults_are_model_neutral() -> Result<(), String> {
    let config = document_extract_audio_config(&|_| None)?;

    assert_eq!(config.backend_profile, "hosted-audio-transcript-v1");
    assert_eq!(config.route_provider, None);
    assert_eq!(config.model_routing_mode, WendaoModelRoutingMode::VllmSr);
    assert_eq!(config.vllm_sr_base_url, DEFAULT_WENDAO_VLLM_SR_BASE_URL);
    assert_eq!(config.chunk_duration_ms, 30_000);
    assert_eq!(config.recovery_split_duration_ms, 30_000);
    assert_eq!(config.base_worker_budget, None);
    assert_eq!(config.recovery_worker_budget, None);
    assert_eq!(config.artifact_cache_dir, None);
    assert_eq!(config.audio_bitrate, None);
    assert_eq!(config.speech_segments_jsonl_path, None);
    assert_eq!(config.speech_merge_gap_ms, 500);
    assert_eq!(config.speech_min_window_ms, 0);
    assert_eq!(config.speech_max_window_ms, None);
    assert_eq!(config.speech_boundary_snap_tolerance_ms, 0);
    assert_eq!(config.speech_limit_chunks, 10_000);
    Ok(())
}

#[test]
fn audio_config_parses_model_routing_controls() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_MODEL_ROUTING_MODE" => Some("deterministic".to_owned()),
        "WENDAO_VLLM_SR_BASE_URL" => Some("http://127.0.0.1:8899/".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_ROUTE_PROVIDER" => Some("openrouter".to_owned()),
        _ => None,
    })?;

    assert_eq!(
        config.model_routing_mode,
        WendaoModelRoutingMode::Deterministic
    );
    assert_eq!(config.vllm_sr_base_url, "http://127.0.0.1:8899");
    assert_eq!(config.route_provider.as_deref(), Some("openrouter"));

    assert!(
        document_extract_audio_config(&|key| {
            (key == "WENDAO_MODEL_ROUTING_MODE").then(|| "fallback".to_owned())
        })
        .is_err()
    );
    Ok(())
}

#[test]
fn audio_config_parses_optional_bitrate() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_BITRATE").then(|| "96K".to_owned())
    })?;

    assert_eq!(config.audio_bitrate.as_deref(), Some("96k"));

    let auto_config = document_extract_audio_config(&|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_BITRATE").then(|| "auto".to_owned())
    })?;
    assert_eq!(auto_config.audio_bitrate, None);

    assert!(
        document_extract_audio_config(&|key| {
            (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_BITRATE").then(|| "0k".to_owned())
        })
        .is_err()
    );
    assert!(
        document_extract_audio_config(&|key| {
            (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_BITRATE").then(|| "96kbps".to_owned())
        })
        .is_err()
    );
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
fn audio_config_accepts_explicit_zero_for_neutral_timing_controls() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_BEFORE_MS" => Some("0".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_AFTER_MS" => Some("0".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS" => Some("0".to_owned()),
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_BOUNDARY_SNAP_TOLERANCE_MS" => Some("0".to_owned()),
        _ => None,
    })?;

    assert_eq!(config.context_before_ms, 0);
    assert_eq!(config.context_after_ms, 0);
    assert_eq!(config.speech_min_window_ms, 0);
    assert_eq!(config.speech_boundary_snap_tolerance_ms, 0);
    Ok(())
}

#[test]
fn audio_config_parses_speech_boundary_snap_tolerance() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_BOUNDARY_SNAP_TOLERANCE_MS")
            .then(|| "1000".to_owned())
    })?;

    assert_eq!(config.speech_boundary_snap_tolerance_ms, 1000);

    assert!(
        document_extract_audio_config(&|key| {
            (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_BOUNDARY_SNAP_TOLERANCE_MS")
                .then(|| "invalid".to_owned())
        })
        .is_err()
    );
    Ok(())
}

#[test]
fn audio_config_parses_optional_speech_max_window() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MAX_WINDOW_MS").then(|| "28000".to_owned())
    })?;
    assert_eq!(config.speech_max_window_ms, Some(28_000));

    let auto_config = document_extract_audio_config(&|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MAX_WINDOW_MS").then(|| "auto".to_owned())
    })?;
    assert_eq!(auto_config.speech_max_window_ms, None);

    assert!(
        document_extract_audio_config(&|key| {
            (key == "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MAX_WINDOW_MS").then(|| "0".to_owned())
        })
        .is_err()
    );
    Ok(())
}

#[test]
fn audio_artifact_cache_uses_shared_l2_root_with_route_override() -> Result<(), String> {
    let config = document_extract_audio_config(&|key| match key {
        "PRJ_CACHE_HOME" => Some("/tmp/project-cache".to_owned()),
        _ => None,
    })?;
    assert_eq!(
        config.artifact_cache_dir.as_deref(),
        Some(Path::new("/tmp/project-cache/wendao/artifacts"))
    );

    let shared_root = document_extract_audio_config(&|key| match key {
        "WENDAO_ARTIFACT_CACHE_ROOT" => Some("/tmp/shared-artifacts".to_owned()),
        "PRJ_CACHE_HOME" => Some("/tmp/project-cache".to_owned()),
        _ => None,
    })?;
    assert_eq!(
        shared_root.artifact_cache_dir.as_deref(),
        Some(Path::new("/tmp/shared-artifacts"))
    );

    let route_override = document_extract_audio_config(&|key| match key {
        "WENDAO_DOCUMENT_EXTRACT_AUDIO_ARTIFACT_CACHE_DIR" => Some("/tmp/audio-route".to_owned()),
        "WENDAO_ARTIFACT_CACHE_ROOT" => Some("/tmp/shared-artifacts".to_owned()),
        _ => None,
    })?;
    assert_eq!(
        route_override.artifact_cache_dir.as_deref(),
        Some(Path::new("/tmp/audio-route"))
    );
    Ok(())
}

#[test]
fn parses_ffprobe_duration_as_ceil_milliseconds() -> Result<(), String> {
    assert_eq!(parse_ffprobe_duration_ms("1.2341")?, 1235);
    assert!(parse_ffprobe_duration_ms("0").is_err());
    assert!(parse_ffprobe_duration_ms("not-a-duration").is_err());
    Ok(())
}
