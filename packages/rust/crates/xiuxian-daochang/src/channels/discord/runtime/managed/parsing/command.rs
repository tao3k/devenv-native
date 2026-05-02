//! Discord managed-command parse model.

use crate::channels::discord::DiscordSessionPartition;
use crate::channels::managed_runtime::parsing::{
    OutputFormat, SessionPartitionCommand as SharedSessionPartitionCommand,
    parse_background_prompt, parse_help_command, parse_job_status_command,
    parse_jobs_summary_command, parse_resume_context_command, parse_session_context_budget_command,
    parse_session_context_memory_command, parse_session_context_status_command,
    parse_session_feedback_command, parse_session_mention_command,
};
use crate::channels::telegram::commands::{
    SessionAdminCommand, SessionInjectionCommand, parse_session_admin_command,
    parse_session_injection_command,
};

use super::partition::parse_session_partition_command;
use super::{ResumeCommand, SessionFeedbackCommand, SessionMentionCommand};

pub(in crate::channels::discord::runtime::managed) type CommandOutputFormat = OutputFormat;
pub(in crate::channels::discord::runtime::managed) type SessionPartitionMode =
    DiscordSessionPartition;
pub(in crate::channels::discord::runtime::managed) type SessionPartitionCommand =
    SharedSessionPartitionCommand<SessionPartitionMode>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::channels::discord::runtime::managed) enum ManagedCommand {
    Help(CommandOutputFormat),
    Reset,
    Resume(ResumeCommand),
    SessionStatus(CommandOutputFormat),
    SessionBudget(CommandOutputFormat),
    SessionMemory(CommandOutputFormat),
    SessionAdmin(SessionAdminCommand),
    SessionFeedback(SessionFeedbackCommand),
    SessionInjection(SessionInjectionCommand),
    SessionMention(SessionMentionCommand),
    SessionPartition(SessionPartitionCommand),
    JobStatus {
        job_id: String,
        format: CommandOutputFormat,
    },
    JobsSummary(CommandOutputFormat),
    BackgroundSubmit(String),
}

pub(in crate::channels::discord::runtime::managed) fn parse_managed_command(
    input: &str,
) -> Option<ManagedCommand> {
    if let Some(format) = parse_help_command(input) {
        return Some(ManagedCommand::Help(format));
    }
    if crate::channels::managed_runtime::parsing::is_reset_context_command(input) {
        return Some(ManagedCommand::Reset);
    }
    if let Some(resume) = parse_resume_context_command(input) {
        return Some(ManagedCommand::Resume(resume));
    }
    if let Some(command) = parse_session_partition_command(input) {
        return Some(ManagedCommand::SessionPartition(command));
    }
    if let Some(command) = parse_session_mention_command(input) {
        return Some(ManagedCommand::SessionMention(command));
    }
    if let Some(format) = parse_session_context_status_command(input) {
        return Some(ManagedCommand::SessionStatus(format));
    }
    if let Some(format) = parse_session_context_budget_command(input) {
        return Some(ManagedCommand::SessionBudget(format));
    }
    if let Some(format) = parse_session_context_memory_command(input) {
        return Some(ManagedCommand::SessionMemory(format));
    }
    if let Some(command) = parse_session_admin_command(input) {
        return Some(ManagedCommand::SessionAdmin(command));
    }
    if let Some(command) = parse_session_feedback_command(input) {
        return Some(ManagedCommand::SessionFeedback(command));
    }
    if let Some(command) = parse_session_injection_command(input) {
        return Some(ManagedCommand::SessionInjection(command));
    }
    if let Some(command) = parse_job_status_command(input) {
        return Some(ManagedCommand::JobStatus {
            job_id: command.job_id,
            format: command.format,
        });
    }
    if let Some(format) = parse_jobs_summary_command(input) {
        return Some(ManagedCommand::JobsSummary(format));
    }
    if let Some(prompt) = parse_background_prompt(input) {
        return Some(ManagedCommand::BackgroundSubmit(prompt));
    }
    None
}
