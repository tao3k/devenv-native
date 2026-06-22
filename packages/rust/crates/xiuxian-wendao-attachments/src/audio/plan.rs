//! Shard offset, context-window, and manifest planning for `audio` inputs.

use super::identity::sha256_hex;
use super::types::{
    AudioShardInput, AudioShardManifestItem, AudioShardPlan, AudioShardPlannerInput,
    AudioShardStrategy, AudioSpeechSegment, AudioSpeechWindowPlannerInput,
};

/// Build deterministic shard manifest items for local or hosted audio backends.
///
/// # Errors
///
/// Returns an error when the chunk duration, sample rate, channel count, or
/// audio format is invalid, or when the shard count exceeds `u32::MAX`.
pub fn plan_audio_shards(plan: &AudioShardPlan) -> Result<Vec<AudioShardManifestItem>, String> {
    validate_plan(plan)?;
    planned_windows(plan)?
        .into_iter()
        .enumerate()
        .map(|(index, window)| {
            let chunk_index = u32::try_from(index)
                .map_err(|_| "audio shard count exceeds u32::MAX".to_owned())?;
            let media_window = media_window_for_shard(plan, window)?;
            Ok(AudioShardManifestItem {
                shard_id: audio_shard_id(plan, chunk_index, window, media_window),
                source_id: plan.source.source_id.clone(),
                source_sha256: plan.source.source_sha256.clone(),
                chunk_index,
                start_ms: window.start_ms,
                duration_ms: window.duration_ms,
                media_start_ms: media_window.start,
                media_duration_ms: media_window.duration,
                context_before_ms: media_window.before_context,
                context_after_ms: media_window.after_context,
                sample_rate_hz: plan.sample_rate_hz,
                channels: plan.channels,
                audio_format: normalized_audio_format(plan.audio_format.as_str())?,
                audio_bitrate: normalized_audio_bitrate(plan.audio_bitrate.as_deref())?,
                cache_key: audio_shard_cache_key(plan, chunk_index, window, media_window),
                reading_order_key: format!("{chunk_index:06}.{:012}", window.start_ms),
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
                input
                    .start_offset_ms
                    .saturating_add(u64::from(index).saturating_mul(input.chunk_duration_ms))
            })
            .collect(),
        AudioShardStrategy::Uniform => uniform_offsets_ms(input)?,
    };
    let plan = AudioShardPlan {
        profile: input.profile.clone(),
        source: input.source.clone(),
        chunk_duration_ms: input.chunk_duration_ms,
        start_offsets_ms,
        window_durations_ms: Vec::new(),
        context_before_ms: input.context_before_ms,
        context_after_ms: input.context_after_ms,
        sample_rate_hz: input.sample_rate_hz,
        channels: input.channels,
        audio_format: input.audio_format.clone(),
        audio_bitrate: normalized_audio_bitrate(input.audio_bitrate.as_deref())?,
        strategy: strategy_token(input.strategy).to_owned(),
    };
    validate_plan(&plan)?;
    Ok(plan)
}

/// Build a complete audio shard plan from speech segment timing facts.
///
/// # Errors
///
/// Returns an error when planner input values are invalid, speech segments are
/// empty, or packed windows cannot satisfy the duration limits.
pub fn build_audio_speech_window_plan(
    input: &AudioSpeechWindowPlannerInput,
) -> Result<AudioShardPlan, String> {
    validate_speech_window_input(input)?;
    let windows = pack_speech_segment_windows(
        input.speech_segments.as_slice(),
        input.merge_gap_ms,
        input.min_window_ms,
        input.short_merge_gap_ms.unwrap_or(input.min_window_ms),
        input.max_window_ms,
        input.boundary_snap_tolerance_ms,
    )?;
    let mut windows = windows;
    if windows.len() > input.limit_chunks as usize {
        windows.truncate(input.limit_chunks as usize);
    }
    let plan = AudioShardPlan {
        profile: input.profile.clone(),
        source: input.source.clone(),
        chunk_duration_ms: input.chunk_duration_ms,
        start_offsets_ms: windows.iter().map(|window| window.start_ms).collect(),
        window_durations_ms: windows.iter().map(|window| window.duration_ms).collect(),
        context_before_ms: input.context_before_ms,
        context_after_ms: input.context_after_ms,
        sample_rate_hz: input.sample_rate_hz,
        channels: input.channels,
        audio_format: input.audio_format.clone(),
        audio_bitrate: normalized_audio_bitrate(input.audio_bitrate.as_deref())?,
        strategy: "speech-segments".to_owned(),
    };
    validate_plan(&plan)?;
    Ok(plan)
}

/// Build a short-window recovery plan from selected parent shard indexes.
///
/// # Errors
///
/// Returns an error when the parent plan is invalid, the split duration is
/// zero, a parent index is duplicated, or a parent index is out of range.
pub fn build_audio_recovery_split_plan(
    parent_plan: &AudioShardPlan,
    parent_chunk_indices: &[u32],
    split_duration_ms: u64,
) -> Result<AudioShardPlan, String> {
    if split_duration_ms == 0 {
        return Err("audio recovery split duration must be positive".to_owned());
    }
    validate_unique_parent_indices(parent_chunk_indices)?;
    let parent_windows = planned_windows(parent_plan)?;
    let mut selected = parent_chunk_indices
        .iter()
        .map(|index| {
            let index_usize = *index as usize;
            parent_windows
                .get(index_usize)
                .copied()
                .map(|window| (*index, window))
                .ok_or_else(|| format!("audio recovery parent index {index} is out of range"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    selected.sort_by_key(|(index, window)| (window.start_ms, *index));

    let mut start_offsets_ms = Vec::new();
    let mut window_durations_ms = Vec::new();
    for (_index, parent_window) in selected {
        let mut elapsed_ms = 0_u64;
        while elapsed_ms < parent_window.duration_ms {
            let remaining_ms = parent_window.duration_ms.saturating_sub(elapsed_ms);
            let duration_ms = remaining_ms.min(split_duration_ms);
            start_offsets_ms.push(parent_window.start_ms.saturating_add(elapsed_ms));
            window_durations_ms.push(duration_ms);
            elapsed_ms = elapsed_ms.saturating_add(duration_ms);
        }
    }

    let plan = AudioShardPlan {
        profile: parent_plan.profile.clone(),
        source: parent_plan.source.clone(),
        chunk_duration_ms: split_duration_ms,
        start_offsets_ms,
        window_durations_ms,
        context_before_ms: parent_plan.context_before_ms,
        context_after_ms: parent_plan.context_after_ms,
        sample_rate_hz: parent_plan.sample_rate_hz,
        channels: parent_plan.channels,
        audio_format: parent_plan.audio_format.clone(),
        audio_bitrate: parent_plan.audio_bitrate.clone(),
        strategy: "risk-recovery-split".to_owned(),
    };
    validate_plan(&plan)?;
    Ok(plan)
}

/// Build a short-window recovery plan from selected parent input rows.
///
/// # Errors
///
/// Returns an error when a selected input does not map to exactly one parent
/// plan window or when recovery split planning fails.
pub fn build_audio_recovery_split_plan_for_inputs(
    parent_plan: &AudioShardPlan,
    selected_parent_inputs: &[AudioShardInput],
    split_duration_ms: u64,
) -> Result<AudioShardPlan, String> {
    let parent_windows = planned_windows(parent_plan)?;
    let parent_chunk_indices = selected_parent_inputs
        .iter()
        .map(|input| parent_window_index(parent_windows.as_slice(), input))
        .collect::<Result<Vec<_>, _>>()?;
    build_audio_recovery_split_plan(
        parent_plan,
        parent_chunk_indices.as_slice(),
        split_duration_ms,
    )
}

/// Build a speech-window recovery plan for selected failed parent inputs.
///
/// The caller supplies model-neutral speech timing facts from VAD or another
/// detector. This helper clips those facts to the selected parent windows and
/// then reuses the normal Rust speech-window planner, so recovery materializes
/// only known speech-bearing spans instead of blindly splitting the full failed
/// parent duration.
///
/// # Errors
///
/// Returns an error when the parent plan is invalid, a selected input does not
/// map to exactly one parent window, speech-window options are invalid, or the
/// clipped speech plan is invalid.
pub fn build_audio_recovery_speech_window_plan_for_inputs(
    parent_plan: &AudioShardPlan,
    selected_parent_inputs: &[AudioShardInput],
    speech_window_input: &AudioSpeechWindowPlannerInput,
) -> Result<Option<AudioShardPlan>, String> {
    let input = recovery_speech_window_plan_inputs(
        parent_plan,
        selected_parent_inputs,
        speech_window_input,
    )?;
    input
        .as_ref()
        .map(build_recovery_speech_window_plan)
        .transpose()
}

struct AudioRecoverySpeechWindowPlanInput<'a> {
    parent_plan: &'a AudioShardPlan,
    speech_window_input: &'a AudioSpeechWindowPlannerInput,
    recovery_windows: Vec<AudioShardWindow>,
}

fn recovery_speech_window_plan_inputs<'a>(
    parent_plan: &'a AudioShardPlan,
    selected_parent_inputs: &[AudioShardInput],
    speech_window_input: &'a AudioSpeechWindowPlannerInput,
) -> Result<Option<AudioRecoverySpeechWindowPlanInput<'a>>, String> {
    if selected_parent_inputs.is_empty() {
        return Ok(None);
    }
    validate_recovery_speech_window_request(parent_plan, speech_window_input)?;
    let selected_windows = selected_parent_windows(parent_plan, selected_parent_inputs)?;
    let recovery_windows = capped_recovery_speech_windows(
        speech_window_input.speech_segments.as_slice(),
        selected_windows.as_slice(),
        speech_window_input,
    )?;
    if recovery_windows.is_empty() {
        return Ok(None);
    }
    Ok(Some(AudioRecoverySpeechWindowPlanInput {
        parent_plan,
        speech_window_input,
        recovery_windows,
    }))
}

fn validate_recovery_speech_window_request(
    parent_plan: &AudioShardPlan,
    speech_window_input: &AudioSpeechWindowPlannerInput,
) -> Result<(), String> {
    validate_plan(parent_plan)?;
    validate_speech_window_input(speech_window_input)?;
    if speech_window_input.source != parent_plan.source {
        return Err("audio recovery speech-window source must match parent plan".to_owned());
    }
    Ok(())
}

fn selected_parent_windows(
    parent_plan: &AudioShardPlan,
    selected_parent_inputs: &[AudioShardInput],
) -> Result<Vec<AudioShardWindow>, String> {
    let parent_windows = planned_windows(parent_plan)?;
    let selected_indices = selected_parent_inputs
        .iter()
        .map(|input| parent_window_index(parent_windows.as_slice(), input))
        .collect::<Result<Vec<_>, _>>()?;
    validate_unique_parent_indices(selected_indices.as_slice())?;
    selected_indices
        .iter()
        .map(|index| {
            parent_windows
                .get(*index as usize)
                .copied()
                .ok_or_else(|| "audio recovery parent index is out of range".to_owned())
        })
        .collect()
}

fn capped_recovery_speech_windows(
    segments: &[AudioSpeechSegment],
    selected_windows: &[AudioShardWindow],
    speech_window_input: &AudioSpeechWindowPlannerInput,
) -> Result<Vec<AudioShardWindow>, String> {
    let mut recovery_windows = packed_speech_windows_for_selected_windows(
        segments,
        selected_windows,
        speech_window_input,
    )?;
    if recovery_windows.len() > speech_window_input.limit_chunks as usize {
        recovery_windows.truncate(speech_window_input.limit_chunks as usize);
    }
    Ok(recovery_windows)
}

fn build_recovery_speech_window_plan(
    input: &AudioRecoverySpeechWindowPlanInput<'_>,
) -> Result<AudioShardPlan, String> {
    let parent_plan = input.parent_plan;
    let speech_window_input = input.speech_window_input;
    let plan = AudioShardPlan {
        profile: parent_plan.profile.clone(),
        source: parent_plan.source.clone(),
        chunk_duration_ms: speech_window_input.chunk_duration_ms,
        start_offsets_ms: input
            .recovery_windows
            .iter()
            .map(|window| window.start_ms)
            .collect(),
        window_durations_ms: input
            .recovery_windows
            .iter()
            .map(|window| window.duration_ms)
            .collect(),
        context_before_ms: parent_plan.context_before_ms,
        context_after_ms: parent_plan.context_after_ms,
        sample_rate_hz: parent_plan.sample_rate_hz,
        channels: parent_plan.channels,
        audio_format: parent_plan.audio_format.clone(),
        audio_bitrate: parent_plan.audio_bitrate.clone(),
        strategy: "speech-segments".to_owned(),
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

fn clipped_speech_segments_for_windows(
    segments: &[AudioSpeechSegment],
    windows: &[AudioShardWindow],
) -> Result<Vec<AudioSpeechSegment>, String> {
    let mut clipped = Vec::new();
    for segment in segments {
        if segment.duration_ms == 0 {
            return Err("audio speech segment duration must be positive".to_owned());
        }
        let segment_end_ms = segment
            .start_ms
            .checked_add(segment.duration_ms)
            .ok_or_else(|| "audio speech segment exceeds u64::MAX".to_owned())?;
        for window in windows {
            let window_end_ms = checked_window_end(*window)?;
            let start_ms = segment.start_ms.max(window.start_ms);
            let end_ms = segment_end_ms.min(window_end_ms);
            if end_ms > start_ms {
                clipped.push(AudioSpeechSegment {
                    index: segment.index,
                    start_ms,
                    duration_ms: end_ms - start_ms,
                });
            }
        }
    }
    clipped.sort_by_key(|segment| (segment.start_ms, segment.index));
    Ok(clipped)
}

fn packed_speech_windows_for_selected_windows(
    segments: &[AudioSpeechSegment],
    windows: &[AudioShardWindow],
    input: &AudioSpeechWindowPlannerInput,
) -> Result<Vec<AudioShardWindow>, String> {
    let mut selected_windows = windows.to_vec();
    selected_windows.sort_by_key(|window| (window.start_ms, window.duration_ms));
    let mut packed_windows = Vec::new();
    for window in selected_windows {
        let clipped_segments =
            clipped_speech_segments_for_windows(segments, std::slice::from_ref(&window))?;
        if clipped_segments.is_empty() {
            continue;
        }
        let mut parent_packed_windows = pack_speech_segment_windows(
            clipped_segments.as_slice(),
            input.merge_gap_ms,
            input.min_window_ms,
            input.short_merge_gap_ms.unwrap_or(input.min_window_ms),
            input.max_window_ms,
            input.boundary_snap_tolerance_ms,
        )?;
        parent_packed_windows.sort_by_key(|window| (window.start_ms, window.duration_ms));
        packed_windows.extend(parent_packed_windows);
    }
    packed_windows.sort_by_key(|window| (window.start_ms, window.duration_ms));
    Ok(packed_windows)
}

fn validate_unique_parent_indices(parent_chunk_indices: &[u32]) -> Result<(), String> {
    let mut sorted = parent_chunk_indices.to_vec();
    sorted.sort_unstable();
    if let Some(duplicate) = sorted
        .windows(2)
        .find_map(|window| (window[0] == window[1]).then_some(window[0]))
    {
        return Err(format!(
            "audio recovery parent index {duplicate} is duplicated"
        ));
    }
    Ok(())
}

fn parent_window_index(
    parent_windows: &[AudioShardWindow],
    input: &AudioShardInput,
) -> Result<u32, String> {
    let matches = parent_windows
        .iter()
        .enumerate()
        .filter(|(_, window)| {
            window.start_ms == input.start_ms && window.duration_ms == input.duration_ms
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => u32::try_from(*index)
            .map_err(|_| "audio recovery parent index exceeds u32::MAX".to_owned()),
        [] => Err(format!(
            "audio recovery parent input {} does not match a parent plan window",
            input.shard_element_id
        )),
        _ => Err(format!(
            "audio recovery parent input {} matches multiple parent plan windows",
            input.shard_element_id
        )),
    }
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
    if !plan.window_durations_ms.is_empty()
        && plan.window_durations_ms.len() != plan.start_offsets_ms.len()
    {
        return Err("audio window duration count must match shard start offsets".to_owned());
    }
    if plan.window_durations_ms.contains(&0) {
        return Err("audio window duration must be positive".to_owned());
    }
    if plan.sample_rate_hz == 0 {
        return Err("audio sample rate must be positive".to_owned());
    }
    if plan.channels == 0 {
        return Err("audio channel count must be positive".to_owned());
    }
    normalized_audio_format(plan.audio_format.as_str())?;
    normalized_audio_bitrate(plan.audio_bitrate.as_deref())?;
    Ok(())
}

fn validate_speech_window_input(input: &AudioSpeechWindowPlannerInput) -> Result<(), String> {
    if input.limit_chunks == 0 {
        return Err("audio speech window limit must be positive".to_owned());
    }
    if input.chunk_duration_ms == 0 {
        return Err("audio speech chunk duration must be positive".to_owned());
    }
    if input.speech_segments.is_empty() {
        return Err("audio speech segments cannot be empty".to_owned());
    }
    if input.max_window_ms == Some(0) {
        return Err("audio speech max window must be positive".to_owned());
    }
    if input
        .max_window_ms
        .is_some_and(|max_window_ms| input.min_window_ms > max_window_ms)
    {
        return Err("audio speech min window cannot exceed max window".to_owned());
    }
    normalized_audio_format(input.audio_format.as_str())?;
    normalized_audio_bitrate(input.audio_bitrate.as_deref())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AudioShardWindow {
    start_ms: u64,
    duration_ms: u64,
}

fn planned_windows(plan: &AudioShardPlan) -> Result<Vec<AudioShardWindow>, String> {
    if plan.window_durations_ms.is_empty() {
        return Ok(plan
            .start_offsets_ms
            .iter()
            .map(|start_ms| AudioShardWindow {
                start_ms: *start_ms,
                duration_ms: plan.chunk_duration_ms,
            })
            .collect());
    }
    if plan.window_durations_ms.len() != plan.start_offsets_ms.len() {
        return Err("audio window duration count must match shard start offsets".to_owned());
    }
    Ok(plan
        .start_offsets_ms
        .iter()
        .zip(plan.window_durations_ms.iter())
        .map(|(start_ms, duration_ms)| AudioShardWindow {
            start_ms: *start_ms,
            duration_ms: *duration_ms,
        })
        .collect())
}

fn pack_speech_segment_windows(
    segments: &[AudioSpeechSegment],
    merge_gap_ms: u64,
    min_window_ms: u64,
    short_merge_gap_ms: u64,
    max_window_ms: Option<u64>,
    boundary_snap_tolerance_ms: u64,
) -> Result<Vec<AudioShardWindow>, String> {
    let mut sorted_segments = segments.to_vec();
    sorted_segments.sort_by_key(|segment| (segment.start_ms, segment.index));

    let expanded_segments = sorted_segments
        .iter()
        .map(|segment| {
            expand_speech_segment_windows(
                segment,
                max_window_ms,
                min_window_ms,
                boundary_snap_tolerance_ms,
            )
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    expanded_segments
        .into_iter()
        .try_fold(Vec::new(), |windows, segment| {
            merge_packed_speech_window(
                windows,
                segment,
                merge_gap_ms,
                min_window_ms,
                short_merge_gap_ms,
                max_window_ms,
                boundary_snap_tolerance_ms,
            )
        })
}

fn merge_packed_speech_window(
    mut windows: Vec<AudioShardWindow>,
    segment: AudioShardWindow,
    merge_gap_ms: u64,
    min_window_ms: u64,
    short_merge_gap_ms: u64,
    max_window_ms: Option<u64>,
    boundary_snap_tolerance_ms: u64,
) -> Result<Vec<AudioShardWindow>, String> {
    let Some(current) = windows.last_mut() else {
        windows.push(segment);
        return Ok(windows);
    };
    let current_end_ms = checked_window_end(*current)?;
    let segment_end_ms = checked_window_end(segment)?;
    let gap_ms = segment.start_ms.saturating_sub(current_end_ms);
    let merged_end_ms = current_end_ms.max(segment_end_ms);
    let merged_duration_ms = merged_end_ms.saturating_sub(current.start_ms);
    let current_is_short = current.duration_ms < min_window_ms;
    let segment_is_short = segment.duration_ms < min_window_ms;
    let can_short_merge = (current_is_short || segment_is_short) && gap_ms <= short_merge_gap_ms;
    let within_max_window = max_window_ms
        .map(|max_window_ms| speech_window_soft_cap(max_window_ms, boundary_snap_tolerance_ms))
        .transpose()?
        .is_none_or(|max_window_ms| merged_duration_ms <= max_window_ms);
    let can_merge = (gap_ms <= merge_gap_ms || can_short_merge) && within_max_window;
    if can_merge {
        current.duration_ms = merged_duration_ms;
    } else {
        windows.push(segment);
    }
    Ok(windows)
}

fn expand_speech_segment_windows(
    segment: &AudioSpeechSegment,
    max_window_ms: Option<u64>,
    min_window_ms: u64,
    boundary_snap_tolerance_ms: u64,
) -> Result<Vec<AudioShardWindow>, String> {
    if segment.duration_ms == 0 {
        return Err("audio speech segment duration must be positive".to_owned());
    }
    let Some(max_window_ms) = max_window_ms else {
        return Ok(vec![AudioShardWindow {
            start_ms: segment.start_ms,
            duration_ms: segment.duration_ms,
        }]);
    };
    let soft_cap_ms = speech_window_soft_cap(max_window_ms, boundary_snap_tolerance_ms)?;
    if segment.duration_ms <= soft_cap_ms {
        return Ok(vec![AudioShardWindow {
            start_ms: segment.start_ms,
            duration_ms: segment.duration_ms,
        }]);
    }
    if let Some(windows) =
        balanced_long_speech_segment_windows(segment, max_window_ms, min_window_ms)?
    {
        return Ok(windows);
    }
    let mut windows = Vec::new();
    let mut remaining_ms = segment.duration_ms;
    let mut start_ms = segment.start_ms;
    while remaining_ms > 0 {
        let duration_ms = remaining_ms.min(max_window_ms);
        windows.push(AudioShardWindow {
            start_ms,
            duration_ms,
        });
        remaining_ms -= duration_ms;
        start_ms = start_ms
            .checked_add(duration_ms)
            .ok_or_else(|| "audio speech segment exceeds u64::MAX".to_owned())?;
    }
    Ok(windows)
}

fn balanced_long_speech_segment_windows(
    segment: &AudioSpeechSegment,
    max_window_ms: u64,
    min_window_ms: u64,
) -> Result<Option<Vec<AudioShardWindow>>, String> {
    if min_window_ms == 0 {
        return Ok(None);
    }
    let remainder_ms = segment.duration_ms % max_window_ms;
    if remainder_ms == 0 || remainder_ms >= min_window_ms {
        return Ok(None);
    }
    let chunk_count = segment.duration_ms.div_ceil(max_window_ms);
    if chunk_count <= 1 {
        return Ok(None);
    }
    let base_duration_ms = segment.duration_ms / chunk_count;
    let extra_count = segment.duration_ms % chunk_count;
    if base_duration_ms > max_window_ms || base_duration_ms < min_window_ms {
        return Ok(None);
    }
    let mut windows = Vec::new();
    let mut start_ms = segment.start_ms;
    for index in 0..chunk_count {
        let duration_ms = base_duration_ms + u64::from(index < extra_count);
        windows.push(AudioShardWindow {
            start_ms,
            duration_ms,
        });
        start_ms = start_ms
            .checked_add(duration_ms)
            .ok_or_else(|| "audio speech segment exceeds u64::MAX".to_owned())?;
    }
    Ok(Some(windows))
}

fn speech_window_soft_cap(
    max_window_ms: u64,
    boundary_snap_tolerance_ms: u64,
) -> Result<u64, String> {
    max_window_ms
        .checked_add(boundary_snap_tolerance_ms)
        .ok_or_else(|| "audio speech max window tolerance exceeds u64::MAX".to_owned())
}

fn checked_window_end(window: AudioShardWindow) -> Result<u64, String> {
    window
        .start_ms
        .checked_add(window.duration_ms)
        .ok_or_else(|| "audio speech window exceeds u64::MAX".to_owned())
}

fn normalized_audio_format(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("audio format cannot be empty".to_owned());
    }
    Ok(normalized)
}

fn normalized_audio_bitrate(value: Option<&str>) -> Result<Option<String>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let normalized = value.to_ascii_lowercase();
    let (digits, suffix) = normalized
        .trim_end_matches(|ch: char| ch.is_ascii_alphabetic())
        .split_at(
            normalized
                .trim_end_matches(|ch: char| ch.is_ascii_alphabetic())
                .len(),
        );
    if digits.is_empty() || digits.starts_with('0') || !digits.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err("audio bitrate must be a positive bitrate token such as `96k`".to_owned());
    }
    if !suffix.chars().all(|ch| matches!(ch, 'k' | 'm')) {
        return Err("audio bitrate suffix must be `k`, `m`, or omitted".to_owned());
    }
    Ok(Some(normalized))
}

#[derive(Debug, Clone, Copy)]
struct AudioShardMediaWindow {
    start: u64,
    duration: u64,
    before_context: u64,
    after_context: u64,
}

fn media_window_for_shard(
    plan: &AudioShardPlan,
    window: AudioShardWindow,
) -> Result<AudioShardMediaWindow, String> {
    let start_ms = window.start_ms;
    let media_start_ms = start_ms.saturating_sub(plan.context_before_ms);
    let context_before_ms = start_ms - media_start_ms;
    let requested_end_ms = start_ms
        .checked_add(window.duration_ms)
        .and_then(|value| value.checked_add(plan.context_after_ms))
        .ok_or_else(|| "audio shard media window exceeds u64::MAX".to_owned())?;
    let source_end_ms = plan.source.duration_ms.unwrap_or(requested_end_ms);
    let media_end_ms = requested_end_ms.min(source_end_ms);
    let logical_end_ms = start_ms
        .checked_add(window.duration_ms)
        .ok_or_else(|| "audio shard logical window exceeds u64::MAX".to_owned())?;
    let context_after_ms = media_end_ms.saturating_sub(logical_end_ms);
    Ok(AudioShardMediaWindow {
        start: media_start_ms,
        duration: media_end_ms.saturating_sub(media_start_ms),
        before_context: context_before_ms,
        after_context: context_after_ms,
    })
}

fn audio_shard_id(
    plan: &AudioShardPlan,
    chunk_index: u32,
    window: AudioShardWindow,
    media_window: AudioShardMediaWindow,
) -> String {
    sha256_hex(
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            plan.profile,
            plan.source.source_sha256,
            chunk_index,
            window.start_ms,
            window.duration_ms,
            media_window.start,
            media_window.duration,
            plan.sample_rate_hz,
            plan.channels,
            plan.audio_format.trim().to_ascii_lowercase(),
            normalized_audio_bitrate(plan.audio_bitrate.as_deref())
                .ok()
                .flatten()
                .unwrap_or_default()
        )
        .as_bytes(),
    )
}

fn audio_shard_cache_key(
    plan: &AudioShardPlan,
    chunk_index: u32,
    window: AudioShardWindow,
    media_window: AudioShardMediaWindow,
) -> String {
    format!(
        "{}:{}",
        plan.profile,
        audio_shard_id(plan, chunk_index, window, media_window)
    )
}
