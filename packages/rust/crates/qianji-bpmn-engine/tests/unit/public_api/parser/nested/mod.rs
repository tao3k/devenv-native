pub(super) const EMBEDDED_REVIEW_PROCESS_ID: &str =
    "__embedded_subprocess__::main_process::inline_review";
pub(super) const TRANSACTION_PROCESS_ID: &str = "__transaction__::main_process::payment_tx";

mod call_activity;
mod embedded;
mod event_subprocess;
mod transaction;
