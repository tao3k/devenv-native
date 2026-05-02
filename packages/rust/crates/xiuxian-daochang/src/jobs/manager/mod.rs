//! Background job manager: bounded queue, concurrent workers, timeout handling, heartbeat.

mod completion_context;
mod core;
mod types;

pub(crate) use completion_context::append_completion_to_parent_session;
pub use core::JobManager;
pub use types::{
    JobCompletion, JobCompletionKind, JobManagerConfig, JobMetricsSnapshot, JobRecord, JobState,
    JobStatusSnapshot, QueuedJob, TurnRunner, epoch_millis,
};
