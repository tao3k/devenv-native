//! Transaction lifecycle interface seams.

mod boundary;
mod cancel;
mod error;
mod finalize;
mod queue;
mod shell;
mod throw;

pub(super) use cancel::{
    cancel_transaction_boundary_siblings, cancel_transaction_shell, complete_compensation_handler,
    error_transaction_shell, record_completed_compensable_activity, throw_compensation_end_event,
    throw_compensation_intermediate_event, transaction_compensation_is_running,
};
