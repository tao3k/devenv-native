//! Shard offset, context-window, and manifest planning for `audio` inputs.

use super::identity::sha256_hex;
use super::types::{
    AudioShardManifestItem, AudioShardPlan, AudioShardPlannerInput, AudioShardStrategy,
};

/// Build deterministic shard manifest items for local or hosted audio backends.
///
/// # Errors
///
/// Returns an error when the chunk duration, sample rate, channel count, or
/// audio format is invalid, or when the shard count exceeds `u32::MAX`.
pub fn plan_audio_shards(plan: &AudioShardPlan) -> Result<Vec<AudioShardManifestItem>, String> {
    validate_plan(plan)?;
    plan.start_offsets_ms
        .iter()
        .enumerate()
        .map(|(index, start_ms)| {
            let chunk_index = u32::try_from(index)
                .map_err(|_| "audio shard count exceeds u32::MAX".to_owned())?;
            let media_window = media_window_for_shard(plan, *start_ms)?;
            Ok(AudioShardManifestItem {
                shard_id: audio_shard_id(plan, chunk_index, *start_ms, media_window),
                source_id: plan.source.source_id.clone(),
                source_sha256: plan.source.source_sha256.clone(),
                chunk_index,
                start_ms: *start_ms,
                duration_ms: plan.chunk_duration_ms,
                media_start_ms: media_window.start_ms,
                media_duration_ms: media_window.duration_ms,
                context_before_ms: media_window.context_before_ms,
                context_after_ms: media_window.context_after_ms,
                sample_rate_hz: plan.sample_rate_hz,
                channels: plan.channels,
                audio_format: normalized_audio_format(plan.audio_format.as_str())?,
                cache_key: audio_shard_cache_key(plan, chunk_index, *start_ms, media_window),
                reading_order_key: format!("{chunk_index:06}.{start_ms:012}"),
            })
        })
        .collect()
}

/// Build a complete audio shard plan from duration and strategy.
///
/// # Errors
///
/// Returns an error when planner input values are invalid or when the requested
/// strategy cannot be applied without source duration.
pub fn build_audio_shard_plan(input: &AudioShardPlannerInput) -> Result<AudioShardPlan, String> {
    if input.limit_chunks == 0 {
        return Err("audio shard limit must be positive".to_owned());
    }
    if input.chunk_duration_ms == 0 {
        return Err("audio chunk duration must be positive".to_owned());
    }
    let start_offsets_ms = match input.strategy {
        AudioShardStrategy::Head => (0..input.limit_chunks)
            .map(|index| {
                input.start_offset_ms.saturating_add(
                    u64::from(index)
                        .checked_mul(input.chunk_duration_ms)
                        .unwrap_or(u64::MAX),
                )
            })
            .collect(),
        AudioShardStrategy::Uniform => uniform_offsets_ms(input)?,
    };
    let plan = AudioShardPlan {
        profile: input.profile.clone(),
        source: input.source.clone(),
        chunk_duration_ms: input.chunk_duration_ms,
        start_offsets_ms,
        context_before_ms: input.context_before_ms,
        context_after_ms: input.context_after_ms,
        sample_rate_hz: input.sample_rate_hz,
        channels: input.channels,
        audio_format: input.audio_format.clone(),
        strategy: strategy_token(input.strategy).to_owned(),
    };
    validate_plan(&plan)?;
    Ok(plan)
}

fn uniform_offsets_ms(input: &AudioShardPlannerInput) -> Result<Vec<u64>, String> {
    let duration_ms = input
        .source
        .duration_ms
        .ok_or_else(|| "uniform audio shard planning requires source duration".to_owned())?;
    let max_start_ms = input
        .start_offset_ms
        .max(duration_ms.saturating_sub(input.chunk_duration_ms));
    if input.limit_chunks == 1 {
        return Ok(vec![input.start_offset_ms.min(max_start_ms)]);
    }
    let span = max_start_ms.saturating_sub(input.start_offset_ms);
    let denominator = u64::from(input.limit_chunks - 1);
    Ok((0..input.limit_chunks)
        .map(|index| {
            input
                .start_offset_ms
                .saturating_add(span.saturating_mul(u64::from(index)) / denominator)
                .min(max_start_ms)
        })
        .collect())
}

fn strategy_token(strategy: AudioShardStrategy) -> &'static str {
    match strategy {
        AudioShardStrategy::Head => "head",
        AudioShardStrategy::Uniform => "uniform",
    }
}

fn validate_plan(plan: &AudioShardPlan) -> Result<(), String> {
    if plan.profile.trim().is_empty() {
        return Err("audio shard profile cannot be empty".to_owned());
    }
    if plan.source.source_id.trim().is_empty() {
        return Err("audio source id cannot be empty".to_owned());
    }
    if plan.source.source_sha256.trim().is_empty() {
        return Err("audio source SHA-256 cannot be empty".to_owned());
    }
    if plan.chunk_duration_ms == 0 {
        return Err("audio chunk duration must be positive".to_owned());
    }
    if plan.sample_rate_hz == 0 {
        return Err("audio sample rate must be positive".to_owned());
    }
    if plan.channels == 0 {
        return Err("audio channel count must be positive".to_owned());
    }
    normalized_audio_format(plan.audio_format.as_str())?;
    Ok(())
}

fn normalized_audio_format(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("audio format cannot be empty".to_owned());
    }
    Ok(normalized)
}

#[derive(Debug, Clone, Copy)]
struct AudioShardMediaWindow {
    start_ms: u64,
    duration_ms: u64,
    context_before_ms: u64,
    context_after_ms: u64,
}

fn media_window_for_shard(
    plan: &AudioShardPlan,
    start_ms: u64,
) -> Result<AudioShardMediaWindow, String> {
    let media_start_ms = start_ms.saturating_sub(plan.context_before_ms);
    let context_before_ms = start_ms - media_start_ms;
    let requested_end_ms = start_ms
        .checked_add(plan.chunk_duration_ms)
        .and_then(|value| value.checked_add(plan.context_after_ms))
        .ok_or_else(|| "audio shard media window exceeds u64::MAX".to_owned())?;
    let source_end_ms = plan.source.duration_ms.unwrap_or(requested_end_ms);
    let media_end_ms = requested_end_ms.min(source_end_ms);
    let logical_end_ms = start_ms
        .checked_add(plan.chunk_duration_ms)
        .ok_or_else(|| "audio shard logical window exceeds u64::MAX".to_owned())?;
    let context_after_ms = media_end_ms.saturating_sub(logical_end_ms);
    Ok(AudioShardMediaWindow {
        start_ms: media_start_ms,
        duration_ms: media_end_ms.saturating_sub(media_start_ms),
        context_before_ms,
        context_after_ms,
    })
}

fn audio_shard_id(
    plan: &AudioShardPlan,
    chunk_index: u32,
    start_ms: u64,
    media_window: AudioShardMediaWindow,
) -> String {
    sha256_hex(
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            plan.profile,
            plan.source.source_sha256,
            chunk_index,
            start_ms,
            plan.chunk_duration_ms,
            media_window.start_ms,
            media_window.duration_ms,
            plan.sample_rate_hz,
            plan.channels,
            plan.audio_format.trim().to_ascii_lowercase()
        )
        .as_bytes(),
    )
}

fn audio_shard_cache_key(
    plan: &AudioShardPlan,
    chunk_index: u32,
    start_ms: u64,
    media_window: AudioShardMediaWindow,
) -> String {
    format!(
        "{}:{}",
        plan.profile,
        audio_shard_id(plan, chunk_index, start_ms, media_window)
    )
}
