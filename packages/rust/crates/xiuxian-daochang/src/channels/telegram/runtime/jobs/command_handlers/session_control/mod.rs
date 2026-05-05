//! Telegram session-control command handler branch.

mod agenda;
mod events;
mod help;
mod reset;
mod resume;
mod stop;

pub(in super::super) use agenda::try_handle_agenda_command;
pub(super) use events::{
    EVENT_TELEGRAM_COMMAND_CONTROL_ADMIN_REQUIRED_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_RESET_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_RESET_SNAPSHOT_STATE,
    EVENT_TELEGRAM_COMMAND_SESSION_RESUME_DROP_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_RESUME_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_RESUME_STATUS_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_STOP_IDLE_REPLIED, EVENT_TELEGRAM_COMMAND_SESSION_STOP_REPLIED,
    EVENT_TELEGRAM_COMMAND_SLASH_HELP_JSON_REPLIED, EVENT_TELEGRAM_COMMAND_SLASH_HELP_REPLIED,
};
pub(in super::super) use help::try_handle_help_command;
pub(in super::super) use reset::try_handle_reset_context_command;
pub(in super::super) use resume::try_handle_resume_context_command;
pub(in super::super) use stop::try_handle_stop_command;
