//! Telegram session command typed outputs.

use crate::channels::managed_runtime::parsing::{
    FeedbackDirection as SharedSessionFeedbackDirection, OutputFormat as SharedSessionOutputFormat,
    ResumeCommand as SharedResumeContextCommand,
    SessionFeedbackCommand as SharedSessionFeedbackCommand,
    SessionPartitionCommand as SharedSessionPartitionCommand,
};

pub(crate) type SessionOutputFormat = SharedSessionOutputFormat;
pub(crate) type ResumeContextCommand = SharedResumeContextCommand;
pub(crate) type SessionFeedbackDirection = SharedSessionFeedbackDirection;
pub(crate) type SessionFeedbackCommand = SharedSessionFeedbackCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionPartitionMode {
    Chat,
    ChatUser,
    User,
    ChatThreadUser,
}

impl SessionPartitionMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::ChatUser => "chat_user",
            Self::User => "user",
            Self::ChatThreadUser => "chat_thread_user",
        }
    }
}

pub(crate) type SessionPartitionCommand = SharedSessionPartitionCommand<SessionPartitionMode>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionAdminAction {
    List,
    Set(Vec<String>),
    Add(Vec<String>),
    Remove(Vec<String>),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionAdminCommand {
    pub(crate) action: SessionAdminAction,
    pub(crate) format: SessionOutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionInjectionAction {
    Status,
    Clear,
    SetXml(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionInjectionCommand {
    pub(crate) action: SessionInjectionAction,
    pub(crate) format: SessionOutputFormat,
}
