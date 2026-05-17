//! Error types for Qianji control-plane stores and replay.

use crate::{LeaseId, WorkerId};

/// Result alias for control-plane operations.
pub type ControlResult<T> = Result<T, ControlError>;

/// Errors returned by control-plane contracts.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ControlError {
    /// A stable identifier was blank.
    #[error("control-plane identifier `{field}` cannot be blank")]
    BlankId {
        /// Field name that contained a blank id.
        field: &'static str,
    },
    /// A control event could not be applied to a replay view.
    #[error("invalid control event sequence: {message}")]
    InvalidEventSequence {
        /// Replay diagnostic.
        message: String,
    },
    /// A memory store mutex was poisoned.
    #[error("in-memory control store lock `{lock_name}` was poisoned: {message}")]
    LockPoisoned {
        /// Internal lock name.
        lock_name: &'static str,
        /// Backend diagnostic.
        message: String,
    },
    /// A requested lease is not owned by the caller.
    #[error("step lease `{lease_id}` is not owned by worker `{worker_id}`")]
    LeaseNotOwned {
        /// Lease id.
        lease_id: LeaseId,
        /// Worker id.
        worker_id: WorkerId,
    },
}
