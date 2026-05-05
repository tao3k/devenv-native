//! Discord managed session-command branch for admin and injection handling.

mod admin;
mod budget;
mod feedback;
mod injection;
mod memory;
mod mention;
mod partition;
mod status;

pub(super) use admin::handle_session_admin;
pub(super) use budget::handle_session_budget;
pub(super) use feedback::handle_session_feedback;
pub(super) use injection::handle_session_injection;
pub(super) use memory::handle_session_memory;
pub(super) use mention::handle_session_mention;
pub(super) use partition::handle_session_partition;
pub(super) use status::handle_session_status;
