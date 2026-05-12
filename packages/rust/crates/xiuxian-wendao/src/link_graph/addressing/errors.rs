//! `link_graph::addressing::errors` owns Wendao link graph addressing errors behavior.

use thiserror::Error;

/// Resolve error types for dual-index addressing.
#[derive(Debug, Clone, Error)]
pub enum ResolveError {
    /// Address not found.
    #[error("address '{address}' not found in document '{doc_id}'")]
    NotFound {
        /// Requested address.
        address: String,
        /// Document ID.
        doc_id: String,
    },
    /// Unsupported address type for the given mode.
    #[error("unsupported address type for mode")]
    UnsupportedAddress,
}

/// Error during content modification.
#[derive(Debug, Clone, Error)]
pub enum ModificationError {
    /// Byte range is out of bounds.
    #[error("byte range {start:?}-{end:?} out of bounds (content length: {content_len})")]
    ByteRangeOutOfBounds {
        /// Start byte offset.
        start: usize,
        /// End byte offset.
        end: usize,
        /// Total content length.
        content_len: usize,
    },
    /// Content hash verification failed.
    #[error("content hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        /// Expected hash value.
        expected: String,
        /// Actual hash value.
        actual: String,
    },
    /// Byte range not available.
    #[error("byte range not available for node")]
    NoByteRange,
    /// Signed delta exceeded the supported `i64` range.
    #[error("signed delta overflow while comparing lengths {lhs} and {rhs}")]
    DeltaOverflow {
        /// Left-hand length operand.
        lhs: usize,
        /// Right-hand length operand.
        rhs: usize,
    },
    /// Adjusting a `usize` position by a signed delta overflowed.
    #[error("range adjustment overflow for base {base} with delta {delta}")]
    RangeAdjustmentOverflow {
        /// Original base value.
        base: usize,
        /// Signed delta to apply.
        delta: i64,
    },
}
