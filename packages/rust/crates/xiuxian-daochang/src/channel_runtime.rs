//! Channel runtime mode selection shared by CLI and tests.

use clap::ValueEnum;

/// Runtime transport mode for the Telegram channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TelegramChannelMode {
    /// Receive updates by long polling.
    Polling,
    /// Receive updates through the webhook server.
    Webhook,
}

/// Channel provider selected for the current runtime process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ChannelProvider {
    /// Telegram channel runtime.
    Telegram,
    /// Discord channel runtime.
    Discord,
}

/// Runtime ingress mode for the Discord channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DiscordRuntimeMode {
    /// Connect directly to the Discord gateway.
    Gateway,
    /// Accept events from an external ingress layer.
    Ingress,
}

/// Deduplication backend used by webhook receivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WebhookDedupBackendMode {
    /// Keep dedup state in local process memory.
    Memory,
    /// Keep dedup state in Valkey.
    Valkey,
}
