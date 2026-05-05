//! Discord managed command-dispatch branch for control, jobs, and sessions.

mod control;
mod dispatch;
mod jobs;
mod session;

pub(crate) use dispatch::handle_inbound_managed_command;
