//! Byte-range mutation helpers for link-graph addressed documents.

use crate::link_graph::models::PageIndexNode;
use blake3::Hasher;

use super::errors::ModificationError;

/// Result of a content modification operation.
#[derive(Debug, Clone)]
pub struct ModificationResult {
    /// The new content after modification.
    pub new_content: String,
    /// Number of bytes added (positive) or removed (negative).
    pub byte_delta: i64,
    /// Number of lines added (positive) or removed (negative).
    pub line_delta: i64,
    /// The new content hash after modification.
    pub new_hash: String,
}

struct ReplacementMetrics {
    byte_delta: i64,
    line_delta: i64,
    new_capacity: usize,
}

fn signed_len_delta(lhs: usize, rhs: usize) -> Result<i64, ModificationError> {
    let lhs_i64 = i64::try_from(lhs).map_err(|_| ModificationError::DeltaOverflow { lhs, rhs })?;
    let rhs_i64 = i64::try_from(rhs).map_err(|_| ModificationError::DeltaOverflow { lhs, rhs })?;
    lhs_i64
        .checked_sub(rhs_i64)
        .ok_or(ModificationError::DeltaOverflow { lhs, rhs })
}

fn apply_signed_delta(base: usize, delta: i64) -> Result<usize, ModificationError> {
    if delta >= 0 {
        let magnitude = usize::try_from(delta)
            .map_err(|_| ModificationError::RangeAdjustmentOverflow { base, delta })?;
        base.checked_add(magnitude)
            .ok_or(ModificationError::RangeAdjustmentOverflow { base, delta })
    } else {
        let magnitude = match usize::try_from(delta.unsigned_abs()) {
            Ok(magnitude) => magnitude,
            Err(_) => usize::MAX,
        };
        Ok(base.saturating_sub(magnitude))
    }
}

/// Replace content at a specific byte range.
///
/// This is the core primitive for atomic section modifications.
/// It replaces the content between `byte_start` and `byte_end` with `new_text`.
///
/// # Arguments
///
/// * `content` - The original document content
/// * `byte_start` - Start byte offset (inclusive)
/// * `byte_end` - End byte offset (exclusive)
/// * `new_text` - The replacement text
/// * `expected_hash` - Optional content hash to verify before modification
///
/// # Returns
///
/// The modification result with new content and deltas, or an error.
///
/// # Errors
///
/// Returns [`ModificationError::ByteRangeOutOfBounds`] when the byte range is invalid,
/// [`ModificationError::HashMismatch`] when the provided hash does not match the target slice,
/// and overflow variants when the signed delta cannot be represented safely.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn replace_byte_range(
    content: &str,
    byte_start: usize,
    byte_end: usize,
    new_text: &str,
    expected_hash: Option<&str>,
) -> Result<ModificationResult, ModificationError> {
    validate_byte_range(content, byte_start, byte_end)?;
    verify_expected_hash(content, byte_start, byte_end, expected_hash)?;

    let metrics = replacement_metrics(content, byte_start, byte_end, new_text)?;
    let new_content = build_replaced_content(content, byte_start, byte_end, new_text, &metrics);

    let new_hash = compute_hash(new_text);

    Ok(ModificationResult {
        new_content,
        byte_delta: metrics.byte_delta,
        line_delta: metrics.line_delta,
        new_hash,
    })
}

fn validate_byte_range(
    content: &str,
    byte_start: usize,
    byte_end: usize,
) -> Result<(), ModificationError> {
    let content_len = content.len();
    if byte_start > content_len || byte_end > content_len || byte_start > byte_end {
        Err(ModificationError::ByteRangeOutOfBounds {
            start: byte_start,
            end: byte_end,
            content_len,
        })
    } else {
        Ok(())
    }
}

fn verify_expected_hash(
    content: &str,
    byte_start: usize,
    byte_end: usize,
    expected_hash: Option<&str>,
) -> Result<(), ModificationError> {
    let Some(expected) = expected_hash else {
        return Ok(());
    };

    let actual = compute_hash(&content[byte_start..byte_end]);
    if actual == expected {
        Ok(())
    } else {
        Err(ModificationError::HashMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}

fn replacement_metrics(
    content: &str,
    byte_start: usize,
    byte_end: usize,
    new_text: &str,
) -> Result<ReplacementMetrics, ModificationError> {
    let old_len = byte_end - byte_start;
    let byte_delta = signed_len_delta(new_text.len(), old_len)?;
    let old_lines = content[byte_start..byte_end].lines().count();
    let line_delta = signed_len_delta(new_text.lines().count(), old_lines)?;
    let new_capacity = apply_signed_delta(content.len(), byte_delta)?;
    Ok(ReplacementMetrics {
        byte_delta,
        line_delta,
        new_capacity,
    })
}

fn build_replaced_content(
    content: &str,
    byte_start: usize,
    byte_end: usize,
    new_text: &str,
    metrics: &ReplacementMetrics,
) -> String {
    let mut new_content = String::with_capacity(metrics.new_capacity);
    new_content.push_str(&content[..byte_start]);
    new_content.push_str(new_text);
    new_content.push_str(&content[byte_end..]);
    new_content
}

/// Update a section's content using its byte range.
///
/// This function provides a higher-level interface for section updates.
/// It handles the byte range extraction and verification.
///
/// # Errors
///
/// Returns [`ModificationError::NoByteRange`] when the node lacks byte metadata and forwards any
/// replacement failure from [`replace_byte_range`].
pub fn update_section_content(
    content: &str,
    node: &PageIndexNode,
    new_text: &str,
) -> Result<ModificationResult, ModificationError> {
    let (byte_start, byte_end) = node
        .metadata
        .byte_range
        .ok_or(ModificationError::NoByteRange)?;

    replace_byte_range(
        content,
        byte_start,
        byte_end,
        new_text,
        node.metadata.content_hash.as_deref(),
    )
}

/// Compute Blake3 hash for content verification.
pub(super) fn compute_hash(text: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();
    hash.to_hex()[..16].to_string()
}

/// Calculate new line positions after a modification.
///
/// Given the original line range and the modification deltas,
/// compute the new line range for the modified section.
#[must_use]
/// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
pub fn adjust_line_range(
    original_start: usize,
    original_end: usize,
    line_delta: i64,
    modification_line: usize,
) -> (usize, usize) {
    if modification_line <= original_start {
        (
            apply_signed_delta(original_start, line_delta)
                .unwrap_or(1)
                .max(1),
            apply_signed_delta(original_end, line_delta)
                .unwrap_or(1)
                .max(1),
        )
    } else if modification_line <= original_end {
        (
            original_start,
            apply_signed_delta(original_end, line_delta)
                .unwrap_or(1)
                .max(1),
        )
    } else {
        (original_start, original_end)
    }
}
