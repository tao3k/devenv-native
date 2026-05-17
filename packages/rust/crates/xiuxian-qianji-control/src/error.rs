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
    /// A control store mutex was poisoned.
    #[error("control store lock `{lock_name}` was poisoned: {message}")]
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
    /// An event payload could not be encoded or decoded.
    #[error("control codec operation `{operation}` failed: {message}")]
    Codec {
        /// Failing codec operation.
        operation: &'static str,
        /// Backend diagnostic.
        message: String,
    },
    /// A durable or hot-state storage operation failed.
    #[error("control storage operation `{operation}` failed: {message}")]
    Storage {
        /// Failing storage operation.
        operation: &'static str,
        /// Backend diagnostic.
        message: String,
    },
}
