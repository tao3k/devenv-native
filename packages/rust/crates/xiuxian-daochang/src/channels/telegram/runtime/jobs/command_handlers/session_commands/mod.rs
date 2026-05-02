//! Telegram session-command handler branch for session state mutations.

mod events;
mod session_admin;
mod session_budget;
mod session_feedback;
mod session_injection;
mod session_memory;
mod session_partition;
mod session_status;
mod session_updates;
pub(in crate::channels::telegram::runtime::jobs) use session_admin::try_handle_session_admin_command;
pub(in crate::channels::telegram::runtime::jobs) use session_budget::try_handle_session_budget_command;
pub(in crate::channels::telegram::runtime::jobs) use session_feedback::try_handle_session_feedback_command;
pub(in crate::channels::telegram::runtime::jobs) use session_injection::try_handle_session_injection_command;
pub(in crate::channels::telegram::runtime::jobs) use session_memory::try_handle_session_memory_command;
pub(in crate::channels::telegram::runtime::jobs) use session_partition::try_handle_session_partition_command;
pub(in crate::channels::telegram::runtime::jobs) use session_status::try_handle_session_status_command;

pub(super) use events::{
    EVENT_TELEGRAM_COMMAND_CONTROL_ADMIN_REQUIRED_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_ADMIN_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_ADMIN_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_BUDGET_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_BUDGET_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_FEEDBACK_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_FEEDBACK_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_INJECTION_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_INJECTION_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_MEMORY_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_MEMORY_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_PARTITION_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_PARTITION_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_STATUS_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_STATUS_REPLIED,
};
pub(super) use session_updates::{truncate_preview, update_session_admin_users};
