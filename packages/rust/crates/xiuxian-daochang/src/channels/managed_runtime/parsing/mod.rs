//! Managed runtime parsing branch for commands, formats, and matching.

mod commands;
mod matching;
mod normalize;
mod types;

pub(crate) use commands::{
    is_reset_context_command, is_stop_command, parse_background_prompt, parse_help_command,
    parse_job_status_command, parse_jobs_summary_command, parse_resume_context_command,
    parse_session_context_budget_command, parse_session_context_memory_command,
    parse_session_context_status_command, parse_session_feedback_command,
    parse_session_mention_command, parse_session_partition_command,
};
pub(crate) use normalize::{normalize_command_input, slice_original_command_suffix};
pub(crate) use types::{
    FeedbackDirection, JobStatusCommand, OutputFormat, ResumeCommand, SessionFeedbackCommand,
    SessionMentionCommand, SessionMentionMode, SessionPartitionCommand, SessionPartitionModeToken,
    parse_session_partition_mode_token, session_partition_mode_name,
};
