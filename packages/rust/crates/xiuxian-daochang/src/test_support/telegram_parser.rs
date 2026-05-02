//! Telegram command parser helpers exposed for integration tests.

use crate::channels::telegram::commands;

use super::types::{
    JobStatusCommand, OutputFormat, ResumeContextCommand, SessionAdminCommand,
    SessionFeedbackCommand, SessionInjectionCommand, SessionMentionCommand,
    SessionPartitionCommand, job_status_command_from_internal, output_format_from_internal,
    resume_context_command_from_internal, session_admin_command_from_internal,
    session_feedback_command_from_internal, session_injection_command_from_internal,
    session_mention_command_from_internal, session_partition_command_from_internal,
};

#[must_use]
/// Returns whether input parses as an agenda command.
pub fn test_is_agenda_command(input: &str) -> bool {
    commands::is_agenda_command(input)
}

#[must_use]
/// Returns whether input parses as a reset-context command.
pub fn test_is_reset_context_command(input: &str) -> bool {
    commands::is_reset_context_command(input)
}

#[must_use]
/// Returns whether input parses as a stop command.
pub fn test_is_stop_command(input: &str) -> bool {
    commands::is_stop_command(input)
}

/// Parses help command output format.
pub fn test_parse_help_command(input: &str) -> Option<OutputFormat> {
    commands::parse_help_command(input).map(output_format_from_internal)
}

#[must_use]
/// Parses a background prompt command body.
pub fn test_parse_background_prompt(input: &str) -> Option<String> {
    commands::parse_background_prompt(input)
}

/// Parses a job-status command.
pub fn test_parse_job_status_command(input: &str) -> Option<JobStatusCommand> {
    commands::parse_job_status_command(input).map(job_status_command_from_internal)
}

/// Parses a jobs-summary command.
pub fn test_parse_jobs_summary_command(input: &str) -> Option<OutputFormat> {
    commands::parse_jobs_summary_command(input).map(output_format_from_internal)
}

/// Parses a session-context status command.
pub fn test_parse_session_context_status_command(input: &str) -> Option<OutputFormat> {
    commands::parse_session_context_status_command(input).map(output_format_from_internal)
}

/// Parses a session-context budget command.
pub fn test_parse_session_context_budget_command(input: &str) -> Option<OutputFormat> {
    commands::parse_session_context_budget_command(input).map(output_format_from_internal)
}

/// Parses a session-context memory command.
pub fn test_parse_session_context_memory_command(input: &str) -> Option<OutputFormat> {
    commands::parse_session_context_memory_command(input).map(output_format_from_internal)
}

/// Parses a resume-context command.
pub fn test_parse_resume_context_command(input: &str) -> Option<ResumeContextCommand> {
    commands::parse_resume_context_command(input).map(resume_context_command_from_internal)
}

/// Parses a session feedback command.
pub fn test_parse_session_feedback_command(input: &str) -> Option<SessionFeedbackCommand> {
    commands::parse_session_feedback_command(input).map(session_feedback_command_from_internal)
}

#[must_use]
/// Parses a session partition command.
pub fn test_parse_session_partition_command(input: &str) -> Option<SessionPartitionCommand> {
    commands::parse_session_partition_command(input)
        .map(|command| session_partition_command_from_internal(&command))
}

/// Parses a session mention command.
pub fn test_parse_session_mention_command(input: &str) -> Option<SessionMentionCommand> {
    crate::channels::managed_runtime::parsing::parse_session_mention_command(input)
        .map(session_mention_command_from_internal)
}

/// Parses a session administrator command.
pub fn test_parse_session_admin_command(input: &str) -> Option<SessionAdminCommand> {
    commands::parse_session_admin_command(input).map(session_admin_command_from_internal)
}

/// Parses a session prompt-injection command.
pub fn test_parse_session_injection_command(input: &str) -> Option<SessionInjectionCommand> {
    commands::parse_session_injection_command(input).map(session_injection_command_from_internal)
}
