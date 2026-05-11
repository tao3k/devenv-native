//! Error types for file I/O operations.
//!
//! Follows ODF-REP: Library crates use `thiserror` for explicit error enums.

use thiserror::Error;

/// Observed file size and the configured upper bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSizeLimit {
    /// Actual file size in bytes.
    pub bytes: u64,
    /// Maximum allowed file size in bytes.
    pub limit: u64,
}

impl FileSizeLimit {
    /// Create file size limit evidence.
    #[must_use]
    pub const fn new(bytes: u64, limit: u64) -> Self {
        Self { bytes, limit }
    }
}

/// Error types for file I/O operations.
///
/// Each variant represents a specific failure mode in the I/O pipeline.
#[derive(Error, Debug)]
pub enum IoError {
    /// File does not exist.
    #[error("File not found: {0}")]
    NotFound(String),

    /// File exceeds size limit.
    #[error("File too large: {} bytes (limit: {})", .0.bytes, .0.limit)]
    TooLarge(FileSizeLimit),

    /// File contains binary content (NULL bytes detected).
    #[error("Binary file detected")]
    BinaryFile,

    /// Low-level I/O error from std::io.
    #[error("IO error: {0}")]
    System(#[from] std::io::Error),

    /// File watcher backend error.
    #[cfg(feature = "notify")]
    #[error("Watcher error: {0}")]
    Watcher(#[from] notify::Error),

    /// Invalid UTF-8 encoding.
    #[error("UTF-8 decoding error")]
    Encoding,
}

/// Canonical result alias for `xiuxian-io` operations.
pub type Result<T> = std::result::Result<T, IoError>;
