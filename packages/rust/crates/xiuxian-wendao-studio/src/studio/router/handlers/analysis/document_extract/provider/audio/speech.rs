//! Model-neutral speech timing sidecar loading for audio recovery.

use xiuxian_wendao_attachments::audio::{
    AudioShardPlan, AudioSpeechWindowPlannerInput, parse_audio_speech_segments_sidecar,
};

use super::config::AudioDocumentExtractConfig;

pub(crate) fn recovery_speech_window_input_from_config(
    plan: &AudioShardPlan,
    config: &AudioDocumentExtractConfig,
) -> Result<Option<AudioSpeechWindowPlannerInput>, String> {
    let Some(path) = config.speech_segments_jsonl_path.as_ref() else {
        return Ok(None);
    };
    let contents = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read audio speech segment sidecar {}: {error}",
            path.display()
        )
    })?;
    let speech_segments = parse_audio_speech_segments_sidecar(contents.as_str())?;
    Ok(Some(AudioSpeechWindowPlannerInput {
        profile: plan.profile.clone(),
        source: plan.source.clone(),
        chunk_duration_ms: config.recovery_split_duration_ms,
        limit_chunks: config.speech_limit_chunks,
        speech_segments,
        merge_gap_ms: config.speech_merge_gap_ms,
        min_window_ms: config.speech_min_window_ms,
        short_merge_gap_ms: Some(config.speech_merge_gap_ms),
        max_window_ms: Some(config.recovery_split_duration_ms),
        context_before_ms: plan.context_before_ms,
        context_after_ms: plan.context_after_ms,
        sample_rate_hz: plan.sample_rate_hz,
        channels: plan.channels,
        audio_format: plan.audio_format.clone(),
    }))
}
