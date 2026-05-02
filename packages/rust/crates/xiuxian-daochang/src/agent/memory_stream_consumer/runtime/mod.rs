//! Memory stream consumer runtime branch for polling and backoff control.

mod loop_control;
pub(super) mod read_error;
mod read_error_logging;

pub(super) use loop_control::run_consumer_loop;
