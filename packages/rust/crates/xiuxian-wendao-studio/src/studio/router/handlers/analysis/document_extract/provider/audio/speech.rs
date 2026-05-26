//! Model-neutral speech timing sidecar loading for audio planning.

use std::path::Path;

use sha2::Digest;
use xiuxian_wendao_attachments::audio::{
    AudioShardPlan, AudioSpeechSegment, AudioSpeechWindowPlannerInput,
    build_audio_speech_window_plan, parse_audio_speech_segments_sidecar,
};

use super::config::AudioDocumentExtractConfig;

pub(crate) fn base_speech_window_plan_from_config(
    plan: &AudioShardPlan,
    config: &AudioDocumentExtractConfig,
) -> Result<Option<AudioShardPlan>, String> {
    let Some(input) = speech_window_input_from_config(plan, config, config.chunk_duration_ms)?
    else {
        return Ok(None);
    };
    build_audio_speech_window_plan(&input).map(Some)
}

pub(crate) fn recovery_speech_window_input_from_config(
    plan: &AudioShardPlan,
    config: &AudioDocumentExtractConfig,
) -> Result<Option<AudioSpeechWindowPlannerInput>, String> {
    speech_window_input_from_config(plan, config, config.recovery_split_duration_ms)
}

pub(crate) fn configured_speech_segments_sha256_from_config(
    config: &AudioDocumentExtractConfig,
) -> Result<Option<String>, String> {
    let Some(path) = config.speech_segments_jsonl_path.as_ref() else {
        return Ok(None);
    };
    read_speech_segments_sidecar(path).map(|(_, sha256)| Some(sha256))
}

fn speech_window_input_from_config(
    plan: &AudioShardPlan,
    config: &AudioDocumentExtractConfig,
    chunk_duration_ms: u64,
) -> Result<Option<AudioSpeechWindowPlannerInput>, String> {
    let Some(path) = config.speech_segments_jsonl_path.as_ref() else {
        return Ok(None);
    };
    let (speech_segments, _) = read_speech_segments_sidecar(path)?;
    Ok(Some(AudioSpeechWindowPlannerInput {
        profile: plan.profile.clone(),
        source: plan.source.clone(),
        chunk_duration_ms,
        limit_chunks: config.speech_limit_chunks,
        speech_segments,
        merge_gap_ms: config.speech_merge_gap_ms,
        min_window_ms: config.speech_min_window_ms,
        short_merge_gap_ms: None,
        max_window_ms: Some(config.speech_max_window_ms.unwrap_or(chunk_duration_ms)),
        boundary_snap_tolerance_ms: config.speech_boundary_snap_tolerance_ms,
        context_before_ms: plan.context_before_ms,
        context_after_ms: plan.context_after_ms,
        sample_rate_hz: plan.sample_rate_hz,
        channels: plan.channels,
        audio_format: plan.audio_format.clone(),
        audio_bitrate: plan.audio_bitrate.clone(),
    }))
}

fn read_speech_segments_sidecar(path: &Path) -> Result<(Vec<AudioSpeechSegment>, String), String> {
    let contents = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read audio speech segment sidecar {}: {error}",
            path.display()
        )
    })?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(contents.as_bytes());
    let sha256 = format!("{:x}", hasher.finalize());
    let speech_segments = parse_audio_speech_segments_sidecar(contents.as_str())?;
    Ok((speech_segments, sha256))
}
