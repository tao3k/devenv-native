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
        classify_lowercase_message(&lower)
    }
}

fn classify_lowercase_message(lower: &str) -> RepoErrorKind {
    if is_descriptor_pressure_message(lower) {
        return RepoErrorKind::DescriptorPressure;
    }
    if is_transient_network_message(lower) {
        return RepoErrorKind::TransientNetwork;
    }
    if is_auth_failure_message(lower) {
        return RepoErrorKind::AuthFailed;
    }
    if is_revision_not_found_message(lower) {
        return RepoErrorKind::RevisionNotFound;
    }
    if is_remote_misconfigured_message(lower) {
        return RepoErrorKind::RemoteMisconfigured;
    }
    if is_repository_corrupt_message(lower) {
        return RepoErrorKind::RepositoryCorrupt;
    }
    RepoErrorKind::Permanent
}

fn is_descriptor_pressure_message(lower: &str) -> bool {
    lower.contains("too many open files")
}

fn is_transient_network_message(lower: &str) -> bool {
    [
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
}

fn is_auth_failure_message(lower: &str) -> bool {
    lower.contains("authentication required")
        || lower.contains("authentication failed")
        || lower.contains("permission denied")
}

fn is_revision_not_found_message(lower: &str) -> bool {
    lower.contains("reference not found") || lower.contains("git reference not found")
}

fn is_remote_misconfigured_message(lower: &str) -> bool {
    lower.contains("remote")
        && (lower.contains("missing") || lower.contains("mismatch") || lower.contains("invalid"))
}

fn is_repository_corrupt_message(lower: &str) -> bool {
    lower.contains("corrupt") || lower.contains("invalid object")
}
