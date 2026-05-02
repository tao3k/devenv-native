//! Memory recall state branch for snapshots and session persistence.

mod agent_ops;
mod storage;
mod test_api;
mod types;

use super::Agent;

pub(crate) use test_api::test_snapshot_session_id;
pub(crate) use types::SessionMemoryRecallSnapshotInput;
pub(crate) use types::{
    EMBEDDING_SOURCE_EMBEDDING, EMBEDDING_SOURCE_EMBEDDING_REPAIRED, EMBEDDING_SOURCE_UNKNOWN,
};
pub use types::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};
