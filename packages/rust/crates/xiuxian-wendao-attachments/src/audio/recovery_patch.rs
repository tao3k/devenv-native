//! Precision gates for parent-level audio recovery patches.

use super::merge::merge_audio_shard_results;
use super::types::{AudioShardInput, AudioShardResult, AudioShardResultStatus};
use std::collections::{HashMap, HashSet};

/// Thresholds for accepting a short-window recovery patch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioRecoveryPatchGateOptions {
    /// Maximum allowed Chinese-ratio drop against the parent transcript.
    pub max_chinese_ratio_drop: f64,
    /// Minimum recovery/parent character ratio.
    pub min_char_ratio: f64,
    /// Maximum recovery/parent character ratio.
    pub max_char_ratio: f64,
    /// Maximum repeated n-gram ratio allowed on any recovery part.
    pub max_part_repeated_ngram_ratio: f64,
}

impl Default for AudioRecoveryPatchGateOptions {
    fn default() -> Self {
        Self {
            max_chinese_ratio_drop: 0.03,
            min_char_ratio: 0.65,
            max_char_ratio: 1.40,
            max_part_repeated_ngram_ratio: 0.35,
        }
    }
}

/// Rust-owned mapping from one parent shard to its candidate recovery shards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioRecoveryPatchCandidate {
    /// Base parent shard element id.
    pub parent_shard_element_id: String,
    /// Recovery shard element ids in timeline order.
    pub recovery_shard_element_ids: Vec<String>,
}

/// Stable patch decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRecoveryPatchDecisionKind {
    /// The recovery text may replace the parent transcript.
    AcceptPatch,
    /// The recovery text is rejected and the parent transcript must remain.
    RejectPatch,
}

/// Text quality metrics used by the recovery patch gate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioRecoveryPatchTextMetrics {
    /// Unicode scalar count after trimming.
    pub transcript_chars: usize,
    /// Ratio of CJK characters among non-space characters.
    pub chinese_ratio: f64,
    /// Share of repeated normalized character trigrams.
    pub repeated_ngram_ratio: f64,
}

/// One parent-level recovery patch decision.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioRecoveryPatchDecision {
    /// Base parent shard element id.
    pub parent_shard_element_id: String,
    /// Recovery shard element ids evaluated for this parent.
    pub recovery_shard_element_ids: Vec<String>,
    /// Accept/reject outcome.
    pub decision: AudioRecoveryPatchDecisionKind,
    /// Machine-readable rejection reasons.
    pub rejection_reasons: Vec<String>,
    /// Parent transcript metrics.
    pub parent_metrics: AudioRecoveryPatchTextMetrics,
    /// Concatenated recovery transcript metrics.
    pub recovery_metrics: AudioRecoveryPatchTextMetrics,
    /// Recovery text accepted for patching when `decision` is `AcceptPatch`.
    pub recovery_text: String,
}

/// Complete recovery patch gate report.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioRecoveryPatchGateReport {
    /// One decision per candidate parent.
    pub decisions: Vec<AudioRecoveryPatchDecision>,
    /// Number of accepted parent patches.
    pub accepted_count: usize,
    /// Number of rejected parent patches.
    pub rejected_count: usize,
}

/// Gate short-window recovery rows before they can patch parent transcript rows.
///
/// # Errors
///
/// Returns an error when parent or recovery result ids are duplicated.
pub fn gate_audio_recovery_patches(
    base_results: &[AudioShardResult],
    recovery_results: &[AudioShardResult],
    candidates: &[AudioRecoveryPatchCandidate],
    options: AudioRecoveryPatchGateOptions,
) -> Result<AudioRecoveryPatchGateReport, String> {
    let base_index = unique_result_index(base_results)?;
    let recovery_index = unique_result_index(recovery_results)?;
    let decisions = candidates
        .iter()
        .map(|candidate| gate_candidate(candidate, &base_index, &recovery_index, options))
        .collect::<Vec<_>>();
    let accepted_count = decisions
        .iter()
        .filter(|decision| decision.decision == AudioRecoveryPatchDecisionKind::AcceptPatch)
        .count();
    Ok(AudioRecoveryPatchGateReport {
        rejected_count: decisions.len().saturating_sub(accepted_count),
        accepted_count,
        decisions,
    })
}

/// Build parent-to-recovery patch candidates from Rust-owned shard timelines.
///
/// # Errors
///
/// Returns an error when a recovery shard cannot be assigned to exactly one
/// parent logical shard window.
pub fn build_audio_recovery_patch_candidates(
    parent_inputs: &[AudioShardInput],
    recovery_inputs: &[AudioShardInput],
) -> Result<Vec<AudioRecoveryPatchCandidate>, String> {
    let mut grouped: HashMap<&str, Vec<&AudioShardInput>> = HashMap::new();
    for recovery in recovery_inputs {
        let parent = unique_parent_for_recovery(parent_inputs, recovery)?;
        grouped
            .entry(parent.shard_element_id.as_str())
            .or_default()
            .push(recovery);
    }

    let mut ordered_parents = parent_inputs.iter().collect::<Vec<_>>();
    ordered_parents.sort_by(|left, right| {
        left.reading_order_key
            .cmp(&right.reading_order_key)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });
    Ok(ordered_parents
        .into_iter()
        .filter_map(|parent| {
            let mut recovery_rows = grouped.remove(parent.shard_element_id.as_str())?;
            recovery_rows.sort_by(|left, right| {
                left.reading_order_key
                    .cmp(&right.reading_order_key)
                    .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
            });
            Some(AudioRecoveryPatchCandidate {
                parent_shard_element_id: parent.shard_element_id.clone(),
                recovery_shard_element_ids: recovery_rows
                    .into_iter()
                    .map(|input| input.shard_element_id.clone())
                    .collect(),
            })
        })
        .collect())
}

/// Merge base results after applying accepted short-window recovery patches.
///
/// # Errors
///
/// Returns an error when recovery patch gating fails or the final audio result
/// merge fails.
pub fn merge_audio_shard_results_with_recovery_patches(
    base_inputs: &[AudioShardInput],
    base_results: &[AudioShardResult],
    recovery_results: &[AudioShardResult],
    candidates: &[AudioRecoveryPatchCandidate],
    options: AudioRecoveryPatchGateOptions,
) -> Result<
    (
        super::merge::AudioShardMergeReport,
        AudioRecoveryPatchGateReport,
    ),
    String,
> {
    let gate_report =
        gate_audio_recovery_patches(base_results, recovery_results, candidates, options)?;
    let patched_results = apply_audio_recovery_patch_decisions(base_results, &gate_report);
    let merge_report = merge_audio_shard_results(base_inputs, patched_results.as_slice())?;
    Ok((merge_report, gate_report))
}

/// Apply accepted recovery patch decisions to base audio result rows.
///
/// This helper is pure and does not re-run patch gating. Use it only with a
/// gate report produced for the same base result set.
#[must_use]
pub fn apply_audio_recovery_patch_decisions(
    base_results: &[AudioShardResult],
    gate_report: &AudioRecoveryPatchGateReport,
) -> Vec<AudioShardResult> {
    let accepted_text_by_parent = gate_report
        .decisions
        .iter()
        .filter(|decision| decision.decision == AudioRecoveryPatchDecisionKind::AcceptPatch)
        .map(|decision| {
            (
                decision.parent_shard_element_id.as_str(),
                decision.recovery_text.as_str(),
            )
        })
        .collect::<HashMap<_, _>>();
    base_results
        .iter()
        .map(|result| {
            if let Some(recovery_text) =
                accepted_text_by_parent.get(result.shard_element_id.as_str())
            {
                let mut patched = result.clone();
                patched.status = AudioShardResultStatus::Succeeded;
                patched.text = Some((*recovery_text).to_owned());
                patched.error_message = None;
                patched
            } else {
                result.clone()
            }
        })
        .collect()
}

fn unique_parent_for_recovery<'a>(
    parent_inputs: &'a [AudioShardInput],
    recovery: &AudioShardInput,
) -> Result<&'a AudioShardInput, String> {
    let matches = parent_inputs
        .iter()
        .filter(|parent| contains_recovery_window(parent, recovery))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [parent] => Ok(parent),
        [] => Err(format!(
            "audio recovery shard {} has no parent logical window",
            recovery.shard_element_id
        )),
        _ => Err(format!(
            "audio recovery shard {} matches multiple parent logical windows",
            recovery.shard_element_id
        )),
    }
}

fn contains_recovery_window(parent: &AudioShardInput, recovery: &AudioShardInput) -> bool {
    let parent_start = u128::from(parent.start_ms);
    let parent_end = parent_start.saturating_add(u128::from(parent.duration_ms));
    let recovery_start = u128::from(recovery.start_ms);
    let recovery_end = recovery_start.saturating_add(u128::from(recovery.duration_ms));
    parent.source_content_hash == recovery.source_content_hash
        && parent.source_path == recovery.source_path
        && parent_start <= recovery_start
        && recovery_end <= parent_end
}

fn gate_candidate(
    candidate: &AudioRecoveryPatchCandidate,
    base_index: &HashMap<&str, &AudioShardResult>,
    recovery_index: &HashMap<&str, &AudioShardResult>,
    options: AudioRecoveryPatchGateOptions,
) -> AudioRecoveryPatchDecision {
    let mut reasons = Vec::new();
    let parent_result = base_index
        .get(candidate.parent_shard_element_id.as_str())
        .copied();
    let (parent_text, parent_has_usable_text) = parent_text_for_gate(parent_result, &mut reasons);
    let mut recovery_parts = Vec::new();
    let mut max_part_repeated_ngram_ratio = 0.0_f64;
    for recovery_id in &candidate.recovery_shard_element_ids {
        let recovery_result = recovery_index.get(recovery_id.as_str()).copied();
        let recovery_text = result_text(recovery_result, "recovery", &mut reasons);
        if !recovery_text.is_empty() {
            max_part_repeated_ngram_ratio = max_part_repeated_ngram_ratio
                .max(audio_recovery_text_metrics(recovery_text).repeated_ngram_ratio);
            recovery_parts.push(recovery_text.to_owned());
        }
    }
    let recovery_text = merge_text_parts(recovery_parts.as_slice());
    let parent_metrics = audio_recovery_text_metrics(parent_text);
    let recovery_metrics = audio_recovery_text_metrics(recovery_text.as_str());

    if candidate.recovery_shard_element_ids.is_empty() {
        reasons.push("missing-recovery-shards".to_owned());
    }
    if parent_has_usable_text {
        if recovery_metrics.repeated_ngram_ratio >= parent_metrics.repeated_ngram_ratio {
            reasons.push("repeat-not-improved".to_owned());
        }
        let chinese_drop = parent_metrics.chinese_ratio - recovery_metrics.chinese_ratio;
        if chinese_drop > options.max_chinese_ratio_drop {
            reasons.push("chinese-ratio-drop".to_owned());
        }
        let char_ratio = count_to_f64(recovery_metrics.transcript_chars)
            / count_to_f64(parent_metrics.transcript_chars.max(1));
        if char_ratio < options.min_char_ratio {
            reasons.push("char-collapse".to_owned());
        }
        if char_ratio > options.max_char_ratio {
            reasons.push("char-expansion".to_owned());
        }
    } else if recovery_metrics.transcript_chars == 0 {
        reasons.push("empty-recovery-text".to_owned());
    }
    if max_part_repeated_ngram_ratio > options.max_part_repeated_ngram_ratio {
        reasons.push("part-repeat-too-high".to_owned());
    }
    reasons.sort();
    reasons.dedup();

    AudioRecoveryPatchDecision {
        parent_shard_element_id: candidate.parent_shard_element_id.clone(),
        recovery_shard_element_ids: candidate.recovery_shard_element_ids.clone(),
        decision: if reasons.is_empty() {
            AudioRecoveryPatchDecisionKind::AcceptPatch
        } else {
            AudioRecoveryPatchDecisionKind::RejectPatch
        },
        rejection_reasons: reasons,
        parent_metrics,
        recovery_metrics,
        recovery_text,
    }
}

fn result_text<'a>(
    result: Option<&'a AudioShardResult>,
    role: &str,
    reasons: &mut Vec<String>,
) -> &'a str {
    let Some(result) = result else {
        reasons.push(format!("missing-{role}-result"));
        return "";
    };
    if result.status != AudioShardResultStatus::Succeeded {
        reasons.push(format!("{role}-result-not-succeeded"));
        return "";
    }
    let text = result.text.as_deref().unwrap_or_default().trim();
    if text.is_empty() {
        reasons.push(format!("empty-{role}-text"));
    }
    text
}

fn parent_text_for_gate<'a>(
    result: Option<&'a AudioShardResult>,
    reasons: &mut Vec<String>,
) -> (&'a str, bool) {
    let Some(result) = result else {
        reasons.push("missing-parent-result".to_owned());
        return ("", false);
    };
    match result.status {
        AudioShardResultStatus::Succeeded => {
            let text = result.text.as_deref().unwrap_or_default().trim();
            if text.is_empty() {
                return ("", false);
            }
            (text, true)
        }
        AudioShardResultStatus::Failed => ("", false),
        AudioShardResultStatus::Skipped => {
            reasons.push("parent-result-skipped".to_owned());
            ("", false)
        }
    }
}

fn unique_result_index(
    results: &[AudioShardResult],
) -> Result<HashMap<&str, &AudioShardResult>, String> {
    let mut index = HashMap::new();
    let mut duplicates = HashSet::new();
    for result in results {
        let shard_id = result.shard_element_id.as_str();
        if index.insert(shard_id, result).is_some() {
            duplicates.insert(result.shard_element_id.clone());
        }
    }
    if !duplicates.is_empty() {
        let mut duplicates = duplicates.into_iter().collect::<Vec<_>>();
        duplicates.sort();
        return Err(format!(
            "duplicate audio recovery patch result ids: {}",
            duplicates.join(", ")
        ));
    }
    Ok(index)
}

fn merge_text_parts(parts: &[String]) -> String {
    let mut output = String::new();
    for part in parts {
        append_with_boundary_dedupe(&mut output, part.trim());
    }
    output
}

fn append_with_boundary_dedupe(output: &mut String, next: &str) {
    if next.is_empty() {
        return;
    }
    if output.is_empty() {
        output.push_str(next);
        return;
    }
    let overlap = largest_boundary_overlap(output, next);
    if !output.ends_with('\n') && overlap == 0 {
        output.push('\n');
    }
    output.push_str(&next[overlap..]);
}

fn largest_boundary_overlap(left: &str, right: &str) -> usize {
    let max_len = left.len().min(right.len());
    for len in (1..=max_len).rev() {
        if !left.is_char_boundary(left.len() - len) || !right.is_char_boundary(len) {
            continue;
        }
        if left[left.len() - len..] == right[..len] {
            return len;
        }
    }
    0
}

pub(crate) fn audio_recovery_text_metrics(text: &str) -> AudioRecoveryPatchTextMetrics {
    let trimmed = text.trim();
    AudioRecoveryPatchTextMetrics {
        transcript_chars: trimmed.chars().count(),
        chinese_ratio: chinese_ratio(trimmed),
        repeated_ngram_ratio: repeated_ngram_ratio(trimmed),
    }
}

fn chinese_ratio(text: &str) -> f64 {
    let mut chars = 0_usize;
    let mut chinese = 0_usize;
    for character in text.chars().filter(|character| !character.is_whitespace()) {
        chars += 1;
        if ('\u{4e00}'..='\u{9fff}').contains(&character) {
            chinese += 1;
        }
    }
    if chars == 0 {
        0.0
    } else {
        count_to_f64(chinese) / count_to_f64(chars)
    }
}

fn repeated_ngram_ratio(text: &str) -> f64 {
    let normalized = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<Vec<_>>();
    if normalized.len() < 3 {
        return 0.0;
    }
    let mut counts: HashMap<[char; 3], usize> = HashMap::new();
    let mut total = 0_usize;
    for window in normalized.windows(3) {
        let key = [window[0], window[1], window[2]];
        *counts.entry(key).or_insert(0) += 1;
        total += 1;
    }
    let repeated = counts
        .values()
        .filter(|count| **count > 1)
        .map(|count| count - 1)
        .sum::<usize>();
    if total == 0 {
        0.0
    } else {
        count_to_f64(repeated) / count_to_f64(total)
    }
}

fn count_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}
