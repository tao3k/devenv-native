//! Telegram Bot API request and response adapters.

/// Chat-action request adapter.
pub mod chat_action;
mod commands;
/// Media upload request adapter.
pub mod media;
/// Shared request builder adapter.
pub mod request;
/// Telegram API response decoder.
pub mod response;
