//! BPMN host-session command facade.
//!
//! Start with `api`; it owns the host-session command entrypoint.

mod api;
mod prepared;
mod request;
mod result;
mod runtime;

pub(crate) use api::run_bpmn_host_session_command;
