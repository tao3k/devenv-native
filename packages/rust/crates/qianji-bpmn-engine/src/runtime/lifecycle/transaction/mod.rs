//! Transaction lifecycle interface seams.

mod cancel;
mod detached;
mod finalize;
mod queue;
mod shell;
mod throw;

pub(super) use cancel::{
    cancel_transaction_shell, complete_compensation_handler,
    complete_detached_compensation_handler, detached_compensation_matches_pending,
    record_completed_compensable_activity, throw_compensation_end_event,
    throw_compensation_end_event_async, throw_compensation_intermediate_event,
    throw_compensation_intermediate_event_async, transaction_compensation_is_running,
};
