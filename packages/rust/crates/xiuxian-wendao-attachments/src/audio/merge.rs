//! Deterministic merge helpers for audio shard worker results.

use super::types::{AudioShardInput, AudioShardResult, AudioShardResultStatus};
use std::collections::{HashMap, HashSet};

/// Deterministic transcript merge output for one audio shard batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioShardMergeReport {
    /// Text merged from successful shard rows in listening order.
    pub text: String,
    /// Number of successful shard rows accepted into `text`.
    pub succeeded_count: usize,
    /// Number of failed shard rows observed.
    pub failed_count: usize,
    /// Number of skipped shard rows observed.
    pub skipped_count: usize,
    /// Input shard ids that had no returned result row.
    pub missing_shard_element_ids: Vec<String>,
    /// Result shard ids with `failed` status.
    pub failed_shard_element_ids: Vec<String>,
    /// Result shard ids with `skipped` status.
    pub skipped_shard_element_ids: Vec<String>,
    /// Result shard ids that appeared more than once.
    pub duplicate_shard_element_ids: Vec<String>,
}

impl AudioShardMergeReport {
    /// Return true when every input shard produced exactly one successful row.
    #[must_use]
    pub fn has_complete_success_coverage(&self) -> bool {
        self.failed_count == 0
            && self.skipped_count == 0
            && self.missing_shard_element_ids.is_empty()
            && self.duplicate_shard_element_ids.is_empty()
    }
}

/// Merge audio shard result rows in stable listening order.
///
/// # Errors
///
/// Returns an error when an input/result row has mismatched source identity,
/// shard fingerprint, or non-plain text MIME type for a successful row.
pub fn merge_audio_shard_results(
    inputs: &[AudioShardInput],
    results: &[AudioShardResult],
) -> Result<AudioShardMergeReport, String> {
    let indexed_results = index_results(results);
    let mut text = String::new();
    let mut succeeded_count = 0;
    let mut failed_shard_element_ids = Vec::new();
    let mut skipped_shard_element_ids = Vec::new();
    let mut missing_shard_element_ids = Vec::new();

    let mut ordered_inputs = inputs.iter().collect::<Vec<_>>();
    ordered_inputs.sort_by(|left, right| {
        left.reading_order_key
            .cmp(&right.reading_order_key)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });

    for input in ordered_inputs {
        let Some(result) = indexed_results.unique.get(input.shard_element_id.as_str()) else {
            missing_shard_element_ids.push(input.shard_element_id.clone());
            continue;
        };
        validate_audio_result_matches_input(input, result)?;
        match result.status {
            AudioShardResultStatus::Succeeded => {
                let result_text = result.text.as_deref().unwrap_or_default().trim();
                if result_text.is_empty() {
                    failed_shard_element_ids.push(input.shard_element_id.clone());
                    continue;
                }
                append_with_boundary_dedupe(&mut text, result_text);
                succeeded_count += 1;
            }
            AudioShardResultStatus::Failed => {
                failed_shard_element_ids.push(input.shard_element_id.clone());
            }
            AudioShardResultStatus::Skipped => {
                skipped_shard_element_ids.push(input.shard_element_id.clone());
            }
        }
    }

    Ok(AudioShardMergeReport {
        text,
        succeeded_count,
        failed_count: failed_shard_element_ids.len(),
        skipped_count: skipped_shard_element_ids.len(),
        missing_shard_element_ids,
        failed_shard_element_ids,
        skipped_shard_element_ids,
        duplicate_shard_element_ids: indexed_results.duplicates,
    })
}

fn validate_audio_result_matches_input(
    input: &AudioShardInput,
    result: &AudioShardResult,
) -> Result<(), String> {
    if input.source_content_hash != result.source_content_hash {
        return Err(format!(
            "audio result source hash mismatch for shard {}",
            input.shard_element_id
        ));
    }
    if input.shard_sha256 != result.shard_sha256 {
        return Err(format!(
            "audio result shard hash mismatch for shard {}",
            input.shard_element_id
        ));
    }
    if input.shard_profile != result.shard_profile
        || input.task_profile != result.task_profile
        || input.backend_profile != result.backend_profile
    {
        return Err(format!(
            "audio result profile mismatch for shard {}",
            input.shard_element_id
        ));
    }
    if result.status == AudioShardResultStatus::Succeeded && result.text_mime_type != "text/plain" {
        return Err(format!(
            "audio result text MIME type mismatch for shard {}",
            input.shard_element_id
        ));
    }
    Ok(())
}

struct IndexedAudioResults<'a> {
    unique: HashMap<&'a str, &'a AudioShardResult>,
    duplicates: Vec<String>,
}

fn index_results(results: &[AudioShardResult]) -> IndexedAudioResults<'_> {
    let mut unique = HashMap::new();
    let mut seen_duplicates = HashSet::new();
    let mut duplicates = Vec::new();
    for result in results {
        let shard_id = result.shard_element_id.as_str();
        if unique.contains_key(shard_id) {
            if seen_duplicates.insert(shard_id) {
                duplicates.push(result.shard_element_id.clone());
            }
            continue;
        }
        unique.insert(shard_id, result);
    }
    duplicates.sort();
    IndexedAudioResults { unique, duplicates }
}

fn append_with_boundary_dedupe(output: &mut String, next: &str) {
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
