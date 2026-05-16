//! Model-neutral selection of audio parent shards that need recovery.

use super::recovery_patch::{AudioRecoveryPatchTextMetrics, audio_recovery_text_metrics};
use super::types::{AudioShardInput, AudioShardResult, AudioShardResultStatus};
use std::collections::{HashMap, HashSet};

/// Thresholds used to select parent shards for short-window recovery.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioRiskParentSelectionOptions {
    /// Maximum number of parent shards selected for recovery.
    pub limit_parents: usize,
    /// Repeated n-gram threshold that marks a transcript as repetition-heavy.
    pub min_repeated_ngram_ratio: f64,
    /// Chinese-ratio threshold below which a Chinese-primary transcript is risky.
    pub max_chinese_ratio: f64,
    /// Character-density threshold below which a transcript is sparse.
    pub max_chars_per_minute: f64,
    /// Request latency threshold that marks a shard as unusually slow.
    pub min_latency_ms: u64,
    /// Keep first and last parent rows in the review set.
    pub include_boundaries: bool,
}

impl Default for AudioRiskParentSelectionOptions {
    fn default() -> Self {
        Self {
            limit_parents: 20,
            min_repeated_ngram_ratio: 0.14,
            max_chinese_ratio: 0.85,
            max_chars_per_minute: 180.0,
            min_latency_ms: 55_000,
            include_boundaries: true,
        }
    }
}

/// Request-level timing facts observed by Rust scheduling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioShardRequestMetric {
    /// Stable shard element id.
    pub shard_element_id: String,
    /// End-to-end wall latency for this shard request.
    pub wall_ms: u64,
}

/// One selected parent shard that should be recovered with shorter windows.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioRiskParentSelection {
    /// Stable shard element id from the base input row.
    pub shard_element_id: String,
    /// Logical shard start offset in milliseconds.
    pub start_ms: u64,
    /// Logical shard duration in milliseconds.
    pub duration_ms: u64,
    /// Text metrics computed from the base transcript.
    pub metrics: AudioRecoveryPatchTextMetrics,
    /// Transcript characters per logical minute.
    pub chars_per_minute: f64,
    /// Optional request latency in milliseconds.
    pub wall_ms: Option<u64>,
    /// Machine-readable selection reasons.
    pub reasons: Vec<String>,
    score: f64,
}

/// Select parent shard inputs that should be reprocessed as short windows.
///
/// # Errors
///
/// Returns an error when input ids, result ids, or metric ids are duplicated,
/// or when the parent selection limit is zero.
pub fn select_audio_risk_parent_shards(
    inputs: &[AudioShardInput],
    results: &[AudioShardResult],
    request_metrics: &[AudioShardRequestMetric],
    options: AudioRiskParentSelectionOptions,
) -> Result<Vec<AudioRiskParentSelection>, String> {
    if options.limit_parents == 0 {
        return Err("audio risk parent selection limit must be positive".to_owned());
    }
    let result_index = unique_result_index(results)?;
    let metric_index = unique_metric_index(request_metrics)?;
    let mut ordered_inputs = inputs.iter().collect::<Vec<_>>();
    ordered_inputs.sort_by(|left, right| {
        left.reading_order_key
            .cmp(&right.reading_order_key)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });

    let last_index = ordered_inputs.len().saturating_sub(1);
    let mut candidates = Vec::new();
    for (offset, input) in ordered_inputs.iter().enumerate() {
        let Some(result) = result_index.get(input.shard_element_id.as_str()).copied() else {
            continue;
        };
        let wall_ms = metric_index
            .get(input.shard_element_id.as_str())
            .map(|metric| metric.wall_ms);
        let is_boundary = offset == 0 || offset == last_index;
        let transcript = result.text.as_deref().unwrap_or_default().trim();
        let metrics = audio_recovery_text_metrics(transcript);
        let chars_per_minute = chars_per_minute(metrics.transcript_chars, input.duration_ms);
        let reasons = risk_reasons(
            &result.status,
            metrics,
            chars_per_minute,
            wall_ms,
            is_boundary,
            options,
        );
        if reasons.is_empty() {
            continue;
        }
        let score = risk_score(
            metrics,
            chars_per_minute,
            wall_ms,
            reasons.as_slice(),
            options,
        );
        candidates.push(AudioRiskParentSelection {
            shard_element_id: input.shard_element_id.clone(),
            start_ms: input.start_ms,
            duration_ms: input.duration_ms,
            metrics,
            chars_per_minute,
            wall_ms,
            reasons,
            score,
        });
    }

    let mut selected = select_with_boundary_reservation(candidates, options.limit_parents);
    for selection in &mut selected {
        selection.score = 0.0;
    }
    Ok(selected)
}

fn select_with_boundary_reservation(
    candidates: Vec<AudioRiskParentSelection>,
    limit: usize,
) -> Vec<AudioRiskParentSelection> {
    let mut selected_by_id: HashMap<String, AudioRiskParentSelection> = candidates
        .iter()
        .filter(|candidate| {
            candidate
                .reasons
                .iter()
                .any(|reason| reason == "timeline-boundary")
        })
        .map(|candidate| (candidate.shard_element_id.clone(), candidate.clone()))
        .collect();
    let remaining_slots = limit.saturating_sub(selected_by_id.len());
    let mut ranked = candidates
        .into_iter()
        .filter(|candidate| !selected_by_id.contains_key(candidate.shard_element_id.as_str()))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.start_ms.cmp(&right.start_ms))
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });
    for candidate in ranked.into_iter().take(remaining_slots) {
        selected_by_id.insert(candidate.shard_element_id.clone(), candidate);
    }
    let mut selected = selected_by_id.into_values().collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        left.start_ms
            .cmp(&right.start_ms)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });
    selected
}

fn risk_reasons(
    status: &AudioShardResultStatus,
    metrics: AudioRecoveryPatchTextMetrics,
    chars_per_minute: f64,
    wall_ms: Option<u64>,
    is_boundary: bool,
    options: AudioRiskParentSelectionOptions,
) -> Vec<String> {
    let mut reasons = Vec::new();
    match status {
        AudioShardResultStatus::Succeeded => {}
        AudioShardResultStatus::Failed => {
            reasons.push("failed-result".to_owned());
            if wall_ms.is_some_and(|value| value >= options.min_latency_ms) {
                reasons.push("high-latency".to_owned());
            }
            if is_boundary && options.include_boundaries {
                reasons.push("timeline-boundary".to_owned());
            }
            return reasons;
        }
        AudioShardResultStatus::Skipped => return reasons,
    }
    if metrics.repeated_ngram_ratio >= options.min_repeated_ngram_ratio {
        reasons.push("high-repetition".to_owned());
    }
    if metrics.chinese_ratio <= options.max_chinese_ratio {
        reasons.push("low-chinese-ratio".to_owned());
    }
    if chars_per_minute <= options.max_chars_per_minute {
        reasons.push("low-text-density".to_owned());
    }
    if wall_ms.is_some_and(|value| value >= options.min_latency_ms) {
        reasons.push("high-latency".to_owned());
    }
    if is_boundary && options.include_boundaries {
        reasons.push("timeline-boundary".to_owned());
    }
    reasons
}

fn risk_score(
    metrics: AudioRecoveryPatchTextMetrics,
    chars_per_minute: f64,
    wall_ms: Option<u64>,
    reasons: &[String],
    options: AudioRiskParentSelectionOptions,
) -> f64 {
    let mut score = count_to_f64(reasons.len());
    score += ((metrics.repeated_ngram_ratio - options.min_repeated_ngram_ratio) * 10.0).max(0.0);
    score += (options.max_chinese_ratio - metrics.chinese_ratio).max(0.0);
    score += ((options.max_chars_per_minute - chars_per_minute)
        / options.max_chars_per_minute.max(1.0))
    .max(0.0);
    if let Some(wall_ms) = wall_ms {
        score += (millis_to_f64(wall_ms.saturating_sub(options.min_latency_ms))
            / millis_to_f64(options.min_latency_ms.max(1)))
        .max(0.0);
    }
    score
}

fn chars_per_minute(transcript_chars: usize, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        return 0.0;
    }
    count_to_f64(transcript_chars) / (millis_to_f64(duration_ms) / 60_000.0)
}

fn count_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn millis_to_f64(value: u64) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
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
    reject_duplicates("audio risk parent result", duplicates)?;
    Ok(index)
}

fn unique_metric_index(
    metrics: &[AudioShardRequestMetric],
) -> Result<HashMap<&str, &AudioShardRequestMetric>, String> {
    let mut index = HashMap::new();
    let mut duplicates = HashSet::new();
    for metric in metrics {
        let shard_id = metric.shard_element_id.as_str();
        if index.insert(shard_id, metric).is_some() {
            duplicates.insert(metric.shard_element_id.clone());
        }
    }
    reject_duplicates("audio risk parent request metric", duplicates)?;
    Ok(index)
}

fn reject_duplicates(label: &str, duplicates: HashSet<String>) -> Result<(), String> {
    if duplicates.is_empty() {
        return Ok(());
    }
    let mut duplicates = duplicates.into_iter().collect::<Vec<_>>();
    duplicates.sort();
    Err(format!("duplicate {label} ids: {}", duplicates.join(", ")))
}
