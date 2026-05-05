//! Parsing helpers for Telegram channel control commands.

mod agenda;
mod background;
mod help;
mod job;
mod session;

pub(crate) use crate::channels::managed_runtime::parsing as shared;

pub(crate) use agenda::is_agenda_command;
pub(crate) use background::parse_background_prompt;
pub(crate) use help::parse_help_command;
pub(crate) use job::{parse_job_status_command, parse_jobs_summary_command};
pub(crate) use session::{
    ResumeContextCommand, SessionAdminAction, SessionAdminCommand, SessionFeedbackDirection,
    SessionInjectionAction, SessionInjectionCommand, SessionPartitionCommand, SessionPartitionMode,
    is_reset_context_command, is_stop_command, parse_resume_context_command,
    parse_session_admin_command, parse_session_context_budget_command,
    parse_session_context_memory_command, parse_session_context_status_command,
    parse_session_feedback_command, parse_session_injection_command,
    parse_session_partition_command,
};
