//! Error taxonomy for repository substrate operations.

use thiserror::Error;

/// Stable error classification for repository substrate operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoErrorKind {
    /// The repository configuration does not provide any usable source.
    MissingSource,
    /// The configured local or managed path is invalid.
    InvalidPath,
    /// A transient network failure occurred and the operation may succeed later.
    TransientNetwork,
    /// The process is under file-descriptor pressure.
    DescriptorPressure,
    /// The checkout lock is currently held by another process.
    LockBusy,
    /// Authentication or credentials were rejected.
    AuthFailed,
    /// The requested revision could not be found.
    RevisionNotFound,
    /// The configured remote is missing or mismatched.
    RemoteMisconfigured,
    /// The repository appears to be corrupt or unreadable.
    RepositoryCorrupt,
    /// The requested behavior is not supported.
    Unsupported,
    /// The failure is permanent or could not be classified more narrowly.
    Permanent,
}

/// Crate-owned error type for repository substrate operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{message}")]
pub struct RepoError {
    /// Stable error classification.
    pub kind: RepoErrorKind,
    /// Human-readable detail.
    pub message: String,
}

impl RepoError {
    /// Creates a new error with the given kind and message.
    #[must_use]
    pub fn new(kind: RepoErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Classifies a backend message into the stable taxonomy.
    #[must_use]
    pub fn classify_message(message: &str) -> RepoErrorKind {
        let lower = message.to_ascii_lowercase();
        if lower.contains("too many open files") {
            RepoErrorKind::DescriptorPressure
        } else if [
            "can't assign requested address",
            "failed to connect",
            "could not connect",
            "timed out",
            "timeout",
            "temporary failure",
            "connection reset",
            "connection refused",
            "connection aborted",
            "network is unreachable",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        {
            RepoErrorKind::TransientNetwork
        } else if lower.contains("authentication required")
            || lower.contains("authentication failed")
            || lower.contains("permission denied")
        {
            RepoErrorKind::AuthFailed
        } else if lower.contains("reference not found") || lower.contains("git reference not found")
        {
            RepoErrorKind::RevisionNotFound
        } else if lower.contains("remote")
            && (lower.contains("missing")
                || lower.contains("mismatch")
                || lower.contains("invalid"))
        {
            RepoErrorKind::RemoteMisconfigured
        } else if lower.contains("corrupt") || lower.contains("invalid object") {
            RepoErrorKind::RepositoryCorrupt
        } else {
            RepoErrorKind::Permanent
        }
    }
}
