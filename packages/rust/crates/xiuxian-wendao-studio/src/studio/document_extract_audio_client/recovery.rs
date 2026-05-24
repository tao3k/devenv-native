//! Recovery planning and merge helpers for the audio shard Flight client.

use std::collections::HashMap;

use super::AudioShardFlightResponse;
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRecoveryPatchGateReport, AudioRecoveryPatchMergeRequest,
    AudioRiskParentSelection, AudioRiskParentSelectionOptions, AudioShardInput,
    AudioShardMergeReport, AudioShardPlan, AudioShardRequestMetric, AudioSpeechWindowPlannerInput,
    build_audio_recovery_patch_candidates, build_audio_recovery_speech_window_plan_for_inputs,
    build_audio_recovery_split_plan_for_inputs, merge_audio_shard_results_with_recovery_patches,
    select_audio_risk_parent_shards,
};

/// Rust-planned short-window recovery work derived from a base response.
#[derive(Debug, Clone)]
pub struct AudioShardRecoveryPlanning {
    /// Selected parent shard inputs in recovery execution order.
    pub selected_parent_inputs: Vec<AudioShardInput>,
    /// Selection facts used to justify recovery.
    pub selections: Vec<AudioRiskParentSelection>,
    /// Rust-owned short-window recovery plan.
    pub recovery_plan: AudioShardPlan,
}

/// Named request for planning a short-window audio recovery pass.
#[derive(Debug, Clone, Copy)]
pub struct AudioShardRecoveryPlanRequest<'a> {
    /// Parent/base shard plan that produced the submitted base inputs.
    pub parent_plan: &'a AudioShardPlan,
    /// Base input rows submitted to the analyzer worker.
    pub inputs: &'a [AudioShardInput],
    /// Optional Rust-observed request latencies for base rows.
    pub request_metrics: &'a [AudioShardRequestMetric],
    /// Risk parent selection thresholds.
    pub selection_options: AudioRiskParentSelectionOptions,
    /// Short-window split duration in milliseconds.
    pub split_duration_ms: u64,
    /// Optional model-neutral speech timing facts used to avoid blind recovery
    /// splits on failed windows with no detected speech.
    pub speech_window_input: Option<&'a AudioSpeechWindowPlannerInput>,
}

impl AudioShardFlightResponse {
    /// Plan a short-window recovery pass from base result quality and scheduler
    /// timing facts.
    ///
    /// # Errors
    ///
    /// Returns an error when risk selection fails, selected parent ids do not
    /// resolve to input rows, or recovery split planning fails.
    pub fn plan_recovery_split(
        &self,
        request: AudioShardRecoveryPlanRequest<'_>,
    ) -> Result<AudioShardRecoveryPlanning, String> {
        let selections = select_audio_risk_parent_shards(
            request.inputs,
            self.results.as_slice(),
            request.request_metrics,
            request.selection_options,
        )?;
        let input_index = unique_input_index(request.inputs)?;
        let selected_pairs = selections
            .iter()
            .map(|selection| {
                input_index
                    .get(selection.shard_element_id.as_str())
                    .copied()
                    .cloned()
                    .map(|input| (selection.clone(), input))
                    .ok_or_else(|| {
                        format!(
                            "audio recovery selected parent {} is not in submitted inputs",
                            selection.shard_element_id
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let selected_pairs = selected_pairs
            .into_iter()
            .filter(|(selection, input)| {
                Self::should_execute_recovery_for_parent(
                    selection,
                    input,
                    request.split_duration_ms,
                    request.speech_window_input.is_some(),
                )
            })
            .collect::<Vec<_>>();
        let selections = selected_pairs
            .iter()
            .map(|(selection, _)| selection.clone())
            .collect::<Vec<_>>();
        let selected_parent_inputs = selected_pairs
            .into_iter()
            .map(|(_, input)| input)
            .collect::<Vec<_>>();
        let recovery_plan = if let Some(speech_window_input) = request.speech_window_input {
            build_audio_recovery_speech_window_plan_for_inputs(
                request.parent_plan,
                selected_parent_inputs.as_slice(),
                speech_window_input,
            )?
            .unwrap_or_else(|| empty_recovery_plan(request.parent_plan, speech_window_input))
        } else {
            build_audio_recovery_split_plan_for_inputs(
                request.parent_plan,
                selected_parent_inputs.as_slice(),
                request.split_duration_ms,
            )?
        };
        Ok(AudioShardRecoveryPlanning {
            selected_parent_inputs,
            selections,
            recovery_plan,
        })
    }

    /// Return whether one selected parent can produce new recovery evidence.
    fn should_execute_recovery_for_parent(
        selection: &AudioRiskParentSelection,
        input: &AudioShardInput,
        split_duration_ms: u64,
        has_speech_window_input: bool,
    ) -> bool {
        has_speech_window_input
            || selection
                .reasons
                .iter()
                .any(|reason| reason == "failed-result")
            || split_duration_ms < input.duration_ms
    }

    /// Merge the response rows after applying accepted short-window recovery
    /// patches.
    ///
    /// # Errors
    ///
    /// Returns an error when recovery shards cannot be mapped to parent inputs,
    /// patch gating fails, or final merge validation fails.
    pub fn merge_with_recovery_for_inputs(
        &self,
        inputs: &[AudioShardInput],
        recovery_inputs: &[AudioShardInput],
        recovery_response: &AudioShardFlightResponse,
        options: AudioRecoveryPatchGateOptions,
    ) -> Result<(AudioShardMergeReport, AudioRecoveryPatchGateReport), String> {
        let candidates = build_audio_recovery_patch_candidates(inputs, recovery_inputs)?;
        merge_audio_shard_results_with_recovery_patches(AudioRecoveryPatchMergeRequest {
            base_inputs: inputs,
            base_results: self.results.as_slice(),
            recovery_results: recovery_response.results.as_slice(),
            candidates: candidates.as_slice(),
            options,
        })
    }
}

pub(crate) fn empty_patch_gate_report() -> AudioRecoveryPatchGateReport {
    AudioRecoveryPatchGateReport {
        decisions: Vec::new(),
        accepted_count: 0,
        rejected_count: 0,
    }
}

fn empty_recovery_plan(
    parent_plan: &AudioShardPlan,
    speech_window_input: &AudioSpeechWindowPlannerInput,
) -> AudioShardPlan {
    AudioShardPlan {
        profile: parent_plan.profile.clone(),
        source: parent_plan.source.clone(),
        chunk_duration_ms: speech_window_input.chunk_duration_ms,
        start_offsets_ms: Vec::new(),
        window_durations_ms: Vec::new(),
        context_before_ms: parent_plan.context_before_ms,
        context_after_ms: parent_plan.context_after_ms,
        sample_rate_hz: parent_plan.sample_rate_hz,
        channels: parent_plan.channels,
        audio_format: parent_plan.audio_format.clone(),
        strategy: "speech-window-recovery-empty".to_owned(),
    }
}

fn unique_input_index(
    inputs: &[AudioShardInput],
) -> Result<HashMap<&str, &AudioShardInput>, String> {
    let mut sorted_ids = inputs
        .iter()
        .map(|input| input.shard_element_id.as_str())
        .collect::<Vec<_>>();
    sorted_ids.sort_unstable();
    let duplicates = sorted_ids
        .windows(2)
        .filter_map(|window| (window[0] == window[1]).then_some(window[0]))
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        return Err(format!(
            "duplicate audio recovery input ids: {}",
            duplicates.join(", ")
        ));
    }
    Ok(inputs
        .iter()
        .map(|input| (input.shard_element_id.as_str(), input))
        .collect())
}
