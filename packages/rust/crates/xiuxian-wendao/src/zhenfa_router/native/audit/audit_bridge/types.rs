//! `zhenfa_router::native::audit::audit_bridge::types` owns Wendao audit audit bridge types behavior.

use serde::{Deserialize, Serialize};

/// Byte range for precise content addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    /// Start byte position (inclusive).
    pub start: usize,
    /// End byte position (exclusive).
    pub end: usize,
}

impl ByteRange {
    /// Create a new byte range.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the length of this range.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this range is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if a byte position is within this range.
    #[must_use]
    pub const fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Check if this range overlaps with another.
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Adjust this range by a delta (for offset correction after edits).
    #[must_use]
    pub fn adjust(&self, delta: isize) -> Self {
        let delta_magnitude = delta.unsigned_abs();
        Self {
            start: if delta >= 0 {
                self.start.saturating_add(delta_magnitude)
            } else {
                self.start.saturating_sub(delta_magnitude)
            },
            end: if delta >= 0 {
                self.end.saturating_add(delta_magnitude)
            } else {
                self.end.saturating_sub(delta_magnitude)
            },
        }
    }

    /// Extract content from a string at this range.
    ///
    /// Returns `None` if the range is out of bounds or invalid UTF-8 boundary.
    #[must_use]
    pub fn extract<'a>(&self, content: &'a str) -> Option<&'a str> {
        let bytes = content.as_bytes();
        if !self.is_valid_for(bytes.len()) {
            return None;
        }
        content.get(self.start..self.end)
    }

    /// Check if this range is valid for a given content length.
    #[must_use]
    pub const fn is_valid_for(&self, len: usize) -> bool {
        self.start <= self.end && self.end <= len
    }
}

impl From<(usize, usize)> for ByteRange {
    fn from((start, end): (usize, usize)) -> Self {
        Self::new(start, end)
    }
}

impl From<ByteRange> for (usize, usize) {
    fn from(range: ByteRange) -> Self {
        (range.start, range.end)
    }
}

impl std::fmt::Display for ByteRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}

/// Result of applying a surgical fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixResult {
    /// Fix was applied successfully.
    Success,
    /// Hash mismatch - file was modified since audit.
    HashMismatch {
        /// Expected hash (from audit time).
        expected: String,
        /// Actual hash (current file state).
        actual: String,
    },
    /// Byte range out of bounds.
    OutOfBounds {
        /// Requested range.
        range: ByteRange,
        /// File size in bytes.
        file_size: usize,
    },
    /// Content at byte range doesn't match expected original.
    ContentMismatch {
        /// Expected original content.
        expected: String,
        /// Actual content at range.
        actual: String,
    },
}

/// Operation mode for a batch fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchFixMode {
    /// Replace existing content in an existing file.
    Replace,
    /// Create a new file with the provided replacement content.
    CreateFile,
}

impl std::fmt::Display for FixResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Fix applied successfully"),
            Self::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "Hash mismatch: expected {}..8, got {}..8",
                    &expected[..8.min(expected.len())],
                    &actual[..8.min(actual.len())]
                )
            }
            Self::OutOfBounds { range, file_size } => {
                write!(f, "Byte range {range} exceeds file size {file_size}")
            }
            Self::ContentMismatch { expected, actual } => {
                write!(
                    f,
                    "Content mismatch at byte range: expected {:?}, got {:?}",
                    expected.chars().take(50).collect::<String>(),
                    actual.chars().take(50).collect::<String>()
                )
            }
        }
    }
}

/// Represents a single fix operation to be applied to a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct BatchFix {
    /// Type of issue being fixed.
    pub issue_type: String,
    /// Document path where the fix should be applied.
    pub doc_path: String,
    /// Line number where the fix should be applied (for diagnostics).
    pub line_number: usize,
    /// Original content to be replaced (for verification and fallback).
    pub original_content: String,
    /// Replacement content (e.g., updated property drawer).
    pub replacement: String,
    /// Confidence score for this fix (0.0 - 1.0).
    pub confidence: f32,
    /// Source location where similar code was found (if applicable).
    pub source_location: Option<String>,
    /// Operation mode for this fix.
    pub mode: BatchFixMode,
    /// SHA-256 hash of file content at audit time.
    pub base_hash: Option<String>,
    /// Precise byte range (start, end) for the content to replace.
    pub byte_range: Option<ByteRange>,
}
