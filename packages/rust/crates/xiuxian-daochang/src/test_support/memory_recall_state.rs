//! Memory-recall state helpers exposed for integration tests.

use crate::Agent;
use crate::agent::memory_recall_state as internal;

use super::TestSupportResult;

pub use crate::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};

/// Canonical embedding-source marker for direct embedding recall.
pub const EMBEDDING_SOURCE_EMBEDDING: &str = internal::EMBEDDING_SOURCE_EMBEDDING;
/// Canonical embedding-source marker for repaired embedding recall.
pub const EMBEDDING_SOURCE_EMBEDDING_REPAIRED: &str = internal::EMBEDDING_SOURCE_EMBEDDING_REPAIRED;
/// Canonical embedding-source marker for unknown recall sources.
pub const EMBEDDING_SOURCE_UNKNOWN: &str = internal::EMBEDDING_SOURCE_UNKNOWN;

#[must_use]
/// Builds the storage key session id for recall snapshots.
pub fn snapshot_session_id(session_id: impl AsRef<str>) -> String {
    internal::test_snapshot_session_id(session_id.as_ref())
}

/// Records a typed memory-recall snapshot for a session.
pub async fn record_memory_recall_snapshot(
    agent: &Agent,
    session_id: impl AsRef<str>,
    snapshot: SessionMemoryRecallSnapshot,
) {
    let session_id = session_id.as_ref().to_string();
    agent
        .test_record_memory_recall_snapshot(&session_id, snapshot)
        .await;
}

/// Append a raw memory-recall snapshot payload for compatibility tests.
///
/// # Errors
///
/// Returns an error when session storage append fails.
pub async fn append_memory_recall_snapshot_payload(
    agent: &Agent,
    session_id: impl AsRef<str>,
    payload: String,
) -> TestSupportResult<()> {
    let session_id = session_id.as_ref().to_string();
    agent
        .test_append_memory_recall_snapshot_payload(&session_id, payload)
        .await
}
