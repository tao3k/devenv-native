//! Session context branch for reset policy, summaries, and bounded windows.

mod backup;
mod clock;
mod ids;
mod test_api;
mod types;
mod window_ops;

use super::Agent;

pub(super) use clock::now_unix_ms;
pub(super) use ids::{backup_metadata_session_id, backup_session_id};
pub(crate) use test_api::test_now_unix_ms;
pub use types::{
    SessionContextMode, SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
};
