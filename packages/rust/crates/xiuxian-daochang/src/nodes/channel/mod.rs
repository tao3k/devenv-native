//! Channel node branch for Discord, Telegram, and runtime guards.

mod command;
mod discord;
mod runtime_guard;
mod telegram;
#[cfg(test)]
#[path = "../../../tests/unit/nodes/channel/embedding_guard.rs"]
mod tests;

pub(crate) use command::{ChannelCommandRequest, run_channel_command};
