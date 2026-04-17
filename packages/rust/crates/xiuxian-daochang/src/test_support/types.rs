//! Test-support mirror types for managed and Telegram command parsing.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedControlCommand {
    Reset,
    ResumeRestore,
    ResumeStatus,
    ResumeDrop,
    SessionAdmin,
    SessionInjection,
    SessionMention,
    SessionPartition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedSlashCommand {
    SessionStatus,
    SessionBudget,
    SessionMemory,
    SessionFeedback,
    JobStatus,
    JobsSummary,
    BackgroundSubmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Dashboard,
    Json,
}

impl OutputFormat {
    #[must_use]
    pub const fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobStatusCommand {
    pub job_id: String,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeContextCommand {
    Restore,
    Status,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFeedbackDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionFeedbackCommand {
    pub direction: SessionFeedbackDirection,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPartitionMode {
    Chat,
    ChatUser,
    User,
    ChatThreadUser,
}

impl SessionPartitionMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::ChatUser => "chat_user",
            Self::User => "user",
            Self::ChatThreadUser => "chat_thread_user",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPartitionCommand {
    pub mode: Option<SessionPartitionMode>,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMentionMode {
    Require,
    Open,
    Inherit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionMentionCommand {
    pub mode: Option<SessionMentionMode>,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionAdminAction {
    List,
    Set(Vec<String>),
    Add(Vec<String>),
    Remove(Vec<String>),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAdminCommand {
    pub action: SessionAdminAction,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionInjectionAction {
    Status,
    Clear,
    SetXml(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInjectionCommand {
    pub action: SessionInjectionAction,
    pub format: OutputFormat,
}

pub(super) const fn managed_control_command_from_internal(
    command: crate::channels::managed_commands::ManagedControlCommand,
) -> ManagedControlCommand {
    match command {
        crate::channels::managed_commands::ManagedControlCommand::Reset => {
            ManagedControlCommand::Reset
        }
        crate::channels::managed_commands::ManagedControlCommand::ResumeRestore => {
            ManagedControlCommand::ResumeRestore
        }
        crate::channels::managed_commands::ManagedControlCommand::ResumeStatus => {
            ManagedControlCommand::ResumeStatus
        }
        crate::channels::managed_commands::ManagedControlCommand::ResumeDrop => {
            ManagedControlCommand::ResumeDrop
        }
        crate::channels::managed_commands::ManagedControlCommand::SessionAdmin => {
            ManagedControlCommand::SessionAdmin
        }
        crate::channels::managed_commands::ManagedControlCommand::SessionInjection => {
            ManagedControlCommand::SessionInjection
        }
        crate::channels::managed_commands::ManagedControlCommand::SessionMention => {
            ManagedControlCommand::SessionMention
        }
        crate::channels::managed_commands::ManagedControlCommand::SessionPartition => {
            ManagedControlCommand::SessionPartition
        }
    }
}

pub(super) const fn managed_slash_command_from_internal(
    command: crate::channels::managed_commands::ManagedSlashCommand,
) -> ManagedSlashCommand {
    match command {
        crate::channels::managed_commands::ManagedSlashCommand::SessionStatus => {
            ManagedSlashCommand::SessionStatus
        }
        crate::channels::managed_commands::ManagedSlashCommand::SessionBudget => {
            ManagedSlashCommand::SessionBudget
        }
        crate::channels::managed_commands::ManagedSlashCommand::SessionMemory => {
            ManagedSlashCommand::SessionMemory
        }
        crate::channels::managed_commands::ManagedSlashCommand::SessionFeedback => {
            ManagedSlashCommand::SessionFeedback
        }
        crate::channels::managed_commands::ManagedSlashCommand::JobStatus => {
            ManagedSlashCommand::JobStatus
        }
        crate::channels::managed_commands::ManagedSlashCommand::JobsSummary => {
            ManagedSlashCommand::JobsSummary
        }
        crate::channels::managed_commands::ManagedSlashCommand::BackgroundSubmit => {
            ManagedSlashCommand::BackgroundSubmit
        }
    }
}

pub(super) const fn output_format_from_internal(
    format: crate::channels::managed_runtime::parsing::OutputFormat,
) -> OutputFormat {
    match format {
        crate::channels::managed_runtime::parsing::OutputFormat::Dashboard => {
            OutputFormat::Dashboard
        }
        crate::channels::managed_runtime::parsing::OutputFormat::Json => OutputFormat::Json,
    }
}

pub(super) fn job_status_command_from_internal(
    command: crate::channels::managed_runtime::parsing::JobStatusCommand,
) -> JobStatusCommand {
    JobStatusCommand {
        job_id: command.job_id,
        format: output_format_from_internal(command.format),
    }
}

pub(super) const fn resume_context_command_from_internal(
    command: crate::channels::managed_runtime::parsing::ResumeCommand,
) -> ResumeContextCommand {
    match command {
        crate::channels::managed_runtime::parsing::ResumeCommand::Restore => {
            ResumeContextCommand::Restore
        }
        crate::channels::managed_runtime::parsing::ResumeCommand::Status => {
            ResumeContextCommand::Status
        }
        crate::channels::managed_runtime::parsing::ResumeCommand::Drop => {
            ResumeContextCommand::Drop
        }
    }
}

pub(super) const fn session_feedback_direction_from_internal(
    direction: crate::channels::managed_runtime::parsing::FeedbackDirection,
) -> SessionFeedbackDirection {
    match direction {
        crate::channels::managed_runtime::parsing::FeedbackDirection::Up => {
            SessionFeedbackDirection::Up
        }
        crate::channels::managed_runtime::parsing::FeedbackDirection::Down => {
            SessionFeedbackDirection::Down
        }
    }
}

pub(super) fn session_feedback_command_from_internal(
    command: crate::channels::managed_runtime::parsing::SessionFeedbackCommand,
) -> SessionFeedbackCommand {
    SessionFeedbackCommand {
        direction: session_feedback_direction_from_internal(command.direction),
        format: output_format_from_internal(command.format),
    }
}

pub(super) const fn session_partition_mode_from_internal(
    mode: crate::channels::telegram::commands::SessionPartitionMode,
) -> SessionPartitionMode {
    match mode {
        crate::channels::telegram::commands::SessionPartitionMode::Chat => {
            SessionPartitionMode::Chat
        }
        crate::channels::telegram::commands::SessionPartitionMode::ChatUser => {
            SessionPartitionMode::ChatUser
        }
        crate::channels::telegram::commands::SessionPartitionMode::User => {
            SessionPartitionMode::User
        }
        crate::channels::telegram::commands::SessionPartitionMode::ChatThreadUser => {
            SessionPartitionMode::ChatThreadUser
        }
    }
}

pub(super) fn session_partition_command_from_internal(
    command: &crate::channels::telegram::commands::SessionPartitionCommand,
) -> SessionPartitionCommand {
    SessionPartitionCommand {
        mode: command.mode.map(session_partition_mode_from_internal),
        format: output_format_from_internal(command.format),
    }
}

pub(super) const fn session_mention_mode_from_internal(
    mode: crate::channels::managed_runtime::parsing::SessionMentionMode,
) -> SessionMentionMode {
    match mode {
        crate::channels::managed_runtime::parsing::SessionMentionMode::Require => {
            SessionMentionMode::Require
        }
        crate::channels::managed_runtime::parsing::SessionMentionMode::Open => {
            SessionMentionMode::Open
        }
        crate::channels::managed_runtime::parsing::SessionMentionMode::Inherit => {
            SessionMentionMode::Inherit
        }
    }
}

pub(super) fn session_mention_command_from_internal(
    command: crate::channels::managed_runtime::parsing::SessionMentionCommand,
) -> SessionMentionCommand {
    SessionMentionCommand {
        mode: command.mode.map(session_mention_mode_from_internal),
        format: output_format_from_internal(command.format),
    }
}

pub(super) fn session_admin_command_from_internal(
    command: crate::channels::telegram::commands::SessionAdminCommand,
) -> SessionAdminCommand {
    let action = match command.action {
        crate::channels::telegram::commands::SessionAdminAction::List => SessionAdminAction::List,
        crate::channels::telegram::commands::SessionAdminAction::Set(values) => {
            SessionAdminAction::Set(values)
        }
        crate::channels::telegram::commands::SessionAdminAction::Add(values) => {
            SessionAdminAction::Add(values)
        }
        crate::channels::telegram::commands::SessionAdminAction::Remove(values) => {
            SessionAdminAction::Remove(values)
        }
        crate::channels::telegram::commands::SessionAdminAction::Clear => SessionAdminAction::Clear,
    };

    SessionAdminCommand {
        action,
        format: output_format_from_internal(command.format),
    }
}

pub(super) fn session_injection_command_from_internal(
    command: crate::channels::telegram::commands::SessionInjectionCommand,
) -> SessionInjectionCommand {
    let action = match command.action {
        crate::channels::telegram::commands::SessionInjectionAction::Status => {
            SessionInjectionAction::Status
        }
        crate::channels::telegram::commands::SessionInjectionAction::Clear => {
            SessionInjectionAction::Clear
        }
        crate::channels::telegram::commands::SessionInjectionAction::SetXml(value) => {
            SessionInjectionAction::SetXml(value)
        }
    };

    SessionInjectionCommand {
        action,
        format: output_format_from_internal(command.format),
    }
}
