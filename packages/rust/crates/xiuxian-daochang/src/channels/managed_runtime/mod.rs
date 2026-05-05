//! Managed channel runtime branch for command parsing, replies, and turns.

pub(crate) mod observability;
pub(crate) mod parsing;
pub(crate) mod queue_mode;
pub(crate) mod replies;
pub(crate) mod session_partition;
pub(crate) mod session_partition_persistence;
pub(crate) mod session_turn_queue;
pub(crate) mod turn;

pub use queue_mode::ForegroundQueueMode;
pub(crate) use turn::{ForegroundTurnOutcome, build_session_id, compose_turn_content};
