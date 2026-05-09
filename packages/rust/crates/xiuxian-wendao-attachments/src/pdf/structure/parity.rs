//! Parity checks between baseline and candidate document structure rows.

use std::collections::{BTreeMap, BTreeSet};

use super::DocumentStructureBlock;

const PROTECTED_BLOCK_TYPES: &[&str] = &["table", "formula", "image", "code"];

/// Protected block counts observed in baseline and candidate structures.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentStructureParityCount {
    pub baseline: usize,
    pub candidate: usize,
}

/// Summary of candidate document structure coverage against a baseline.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentStructureParitySummary {
    pub baseline_block_count: usize,
    pub candidate_block_count: usize,
    pub baseline_page_count: usize,
    pub candidate_page_count: usize,
    pub baseline_text_chars: usize,
    pub candidate_text_chars: usize,
    pub protected_block_counts: BTreeMap<String, DocumentStructureParityCount>,
}

/// # Errors
///
/// Returns an error when the candidate structure cannot prove parity with the
/// Docling-derived baseline: missing baseline pages, lower per-page text
/// coverage, protected block loss, or unstable reading order.
pub fn validate_document_structure_parity(
    baseline: &[DocumentStructureBlock],
    candidate: &[DocumentStructureBlock],
) -> Result<DocumentStructureParitySummary, String> {
    if baseline.is_empty() {
        return Err("structure parity baseline is empty".to_string());
    }
    if candidate.is_empty() {
        return Err("structure parity candidate is empty".to_string());
    }
    validate_reading_order("baseline", baseline)?;
    validate_reading_order("candidate", candidate)?;
    validate_page_coverage(baseline, candidate)?;
    validate_page_text_coverage(baseline, candidate)?;
    validate_protected_block_counts(baseline, candidate)?;
    if candidate.len() < baseline.len() {
        return Err(format!(
            "structure parity candidate has {} blocks, below baseline {}",
            candidate.len(),
            baseline.len()
        ));
    }
    Ok(parity_summary(baseline, candidate))
}

fn validate_reading_order(label: &str, blocks: &[DocumentStructureBlock]) -> Result<(), String> {
    for pair in blocks.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        if structure_order_key(previous) > structure_order_key(current) {
            return Err(format!(
                "structure parity {label} is not sorted: `{}` precedes `{}`",
                previous.block_id, current.block_id
            ));
        }
    }
    Ok(())
}

fn validate_page_coverage(
    baseline: &[DocumentStructureBlock],
    candidate: &[DocumentStructureBlock],
) -> Result<(), String> {
    let baseline_pages = page_set(baseline);
    let candidate_pages = page_set(candidate);
    let missing_pages = baseline_pages
        .difference(&candidate_pages)
        .copied()
        .collect::<Vec<_>>();
    if missing_pages.is_empty() {
        return Ok(());
    }
    Err(format!(
        "structure parity candidate is missing baseline pages: {}",
        missing_pages
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn validate_page_text_coverage(
    baseline: &[DocumentStructureBlock],
    candidate: &[DocumentStructureBlock],
) -> Result<(), String> {
    let baseline_chars = text_chars_by_page(baseline);
    let candidate_chars = text_chars_by_page(candidate);
    for (page, baseline_count) in baseline_chars {
        let candidate_count = candidate_chars.get(&page).copied().unwrap_or_default();
        if candidate_count < baseline_count {
            return Err(format!(
                "structure parity candidate page {page} has {candidate_count} text chars, below baseline {baseline_count}"
            ));
        }
    }
    Ok(())
}

fn validate_protected_block_counts(
    baseline: &[DocumentStructureBlock],
    candidate: &[DocumentStructureBlock],
) -> Result<(), String> {
    let baseline_counts = protected_block_counts(baseline);
    let candidate_counts = protected_block_counts(candidate);
    for block_type in PROTECTED_BLOCK_TYPES {
        let baseline_count = baseline_counts
            .get(*block_type)
            .copied()
            .unwrap_or_default();
        let candidate_count = candidate_counts
            .get(*block_type)
            .copied()
            .unwrap_or_default();
        if candidate_count < baseline_count {
            return Err(format!(
                "structure parity candidate has {candidate_count} `{block_type}` blocks, below baseline {baseline_count}"
            ));
        }
    }
    Ok(())
}

fn parity_summary(
    baseline: &[DocumentStructureBlock],
    candidate: &[DocumentStructureBlock],
) -> DocumentStructureParitySummary {
    let baseline_counts = protected_block_counts(baseline);
    let candidate_counts = protected_block_counts(candidate);
    DocumentStructureParitySummary {
        baseline_block_count: baseline.len(),
        candidate_block_count: candidate.len(),
        baseline_page_count: page_set(baseline).len(),
        candidate_page_count: page_set(candidate).len(),
        baseline_text_chars: text_char_count(baseline),
        candidate_text_chars: text_char_count(candidate),
        protected_block_counts: PROTECTED_BLOCK_TYPES
            .iter()
            .map(|block_type| {
                (
                    (*block_type).to_string(),
                    DocumentStructureParityCount {
                        baseline: baseline_counts
                            .get(*block_type)
                            .copied()
                            .unwrap_or_default(),
                        candidate: candidate_counts
                            .get(*block_type)
                            .copied()
                            .unwrap_or_default(),
                    },
                )
            })
            .collect(),
    }
}

fn structure_order_key(block: &DocumentStructureBlock) -> (i32, &str, i32, &str) {
    (
        block.page_index,
        block.reading_order_key.as_str(),
        block.block_index,
        block.block_id.as_str(),
    )
}

fn page_set(blocks: &[DocumentStructureBlock]) -> BTreeSet<i32> {
    blocks.iter().map(|block| block.page_index).collect()
}

fn text_chars_by_page(blocks: &[DocumentStructureBlock]) -> BTreeMap<i32, usize> {
    let mut by_page = BTreeMap::new();
    for block in blocks {
        *by_page.entry(block.page_index).or_insert(0) += text_char_count_one(block);
    }
    by_page
}

fn text_char_count(blocks: &[DocumentStructureBlock]) -> usize {
    blocks.iter().map(text_char_count_one).sum()
}

fn text_char_count_one(block: &DocumentStructureBlock) -> usize {
    if matches!(block.block_type.as_str(), "docling_json" | "document") {
        return 0;
    }
    block
        .content
        .chars()
        .filter(|character| !character.is_whitespace())
        .count()
}

fn protected_block_counts(blocks: &[DocumentStructureBlock]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for block in blocks {
        let block_type = block.block_type.as_str();
        if PROTECTED_BLOCK_TYPES.contains(&block_type) {
            *counts.entry(block_type).or_insert(0) += 1;
        }
    }
    counts
}
