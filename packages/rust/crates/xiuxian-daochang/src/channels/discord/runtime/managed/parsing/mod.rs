//! Discord managed-command parsing bridges shared managed runtime grammar into Discord-specific session partition modes.

mod command;
mod partition;

pub(super) use crate::channels::managed_runtime::parsing::{
    FeedbackDirection, ResumeCommand, SessionFeedbackCommand, SessionMentionCommand,
    SessionMentionMode,
};
pub(super) use command::{
    CommandOutputFormat, ManagedCommand, SessionPartitionCommand, SessionPartitionMode,
    parse_managed_command,
};
