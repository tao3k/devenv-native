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
pub fn is_agenda_command(input: &str) -> bool {
    commands::is_agenda_command(input)
}

#[must_use]
pub fn is_reset_context_command(input: &str) -> bool {
    commands::is_reset_context_command(input)
}

#[must_use]
pub fn is_stop_command(input: &str) -> bool {
    commands::is_stop_command(input)
}

pub fn parse_help_command(input: &str) -> Option<OutputFormat> {
    commands::parse_help_command(input).map(output_format_from_internal)
}

#[must_use]
pub fn parse_background_prompt(input: &str) -> Option<String> {
    commands::parse_background_prompt(input)
}

pub fn parse_job_status_command(input: &str) -> Option<JobStatusCommand> {
    commands::parse_job_status_command(input).map(job_status_command_from_internal)
}

pub fn parse_jobs_summary_command(input: &str) -> Option<OutputFormat> {
    commands::parse_jobs_summary_command(input).map(output_format_from_internal)
}

pub fn parse_session_context_status_command(input: &str) -> Option<OutputFormat> {
    commands::parse_session_context_status_command(input).map(output_format_from_internal)
}

pub fn parse_session_context_budget_command(input: &str) -> Option<OutputFormat> {
    commands::parse_session_context_budget_command(input).map(output_format_from_internal)
}

pub fn parse_session_context_memory_command(input: &str) -> Option<OutputFormat> {
    commands::parse_session_context_memory_command(input).map(output_format_from_internal)
}

pub fn parse_resume_context_command(input: &str) -> Option<ResumeContextCommand> {
    commands::parse_resume_context_command(input).map(resume_context_command_from_internal)
}

pub fn parse_session_feedback_command(input: &str) -> Option<SessionFeedbackCommand> {
    commands::parse_session_feedback_command(input).map(session_feedback_command_from_internal)
}

#[must_use]
pub fn parse_session_partition_command(input: &str) -> Option<SessionPartitionCommand> {
    commands::parse_session_partition_command(input)
        .map(|command| session_partition_command_from_internal(&command))
}

pub fn parse_session_mention_command(input: &str) -> Option<SessionMentionCommand> {
    crate::channels::managed_runtime::parsing::parse_session_mention_command(input)
        .map(session_mention_command_from_internal)
}

pub fn parse_session_admin_command(input: &str) -> Option<SessionAdminCommand> {
    commands::parse_session_admin_command(input).map(session_admin_command_from_internal)
}

pub fn parse_session_injection_command(input: &str) -> Option<SessionInjectionCommand> {
    commands::parse_session_injection_command(input).map(session_injection_command_from_internal)
}
