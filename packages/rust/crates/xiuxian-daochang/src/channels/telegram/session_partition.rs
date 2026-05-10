//! Telegram session partition strategy for multi-chat isolation.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::config::load_runtime_settings;
use crate::env_parse::lookup_non_empty_env;

/// How incoming Telegram messages are mapped to a logical conversation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TelegramSessionPartition {
    /// Share one session for everyone in the same chat (`chat_id`).
    #[default]
    ChatOnly,
    /// Isolate by `chat_id:user_id` (safe option when per-user separation is required).
    ChatUser,
    /// Share one session for the same user across all chats (`user_id`).
    UserOnly,
    /// Isolate by topic inside forum groups: `chat_id:thread_id:user_id`.
    ChatThreadUser,
}

/// Telegram message-thread identifier used in forum topic sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelegramMessageThreadId(i64);

impl TelegramMessageThreadId {
    /// Build a Telegram message-thread id.
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    fn unwrap_or_default(value: Option<Self>) -> i64 {
        value.map_or(0, |thread| thread.0)
    }
}

impl TelegramSessionPartition {
    /// Resolve partition mode from environment.
    ///
    /// Accepted values:
    /// - `chat` (default)
    /// - `chat_user`
    /// - `user`
    /// - `chat_thread_user`
    #[must_use]
    pub fn from_env() -> Self {
        let settings = load_runtime_settings();
        Self::from_lookup(
            |name| std::env::var(name).ok(),
            settings.telegram.session_partition.as_deref(),
        )
    }

    fn from_lookup<F>(lookup: F, settings_value: Option<&str>) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        if let Some(raw) =
            lookup_non_empty_env(&lookup, "XIUXIAN_DAOCHANG_TELEGRAM_SESSION_PARTITION")
        {
            return if let Ok(mode) = raw.parse() {
                mode
            } else {
                tracing::warn!(
                    env_var = "XIUXIAN_DAOCHANG_TELEGRAM_SESSION_PARTITION",
                    value = %raw,
                    "invalid telegram session partition env value; using configured/default partition"
                );
                Self::parse_settings_or_default(settings_value)
            };
        }
        Self::parse_settings_or_default(settings_value)
    }

    fn parse_settings_or_default(settings_value: Option<&str>) -> Self {
        let Some(raw) = settings_value else {
            return Self::default();
        };
        if let Ok(mode) = raw.parse() {
            mode
        } else {
            tracing::warn!(
                value = %raw,
                "invalid telegram.session_partition in settings; using default partition"
            );
            Self::default()
        }
    }

    /// Build a session key from Telegram identifiers.
    #[must_use]
    pub fn build_session_key(
        self,
        chat_id: impl AsRef<str>,
        user_identity: impl AsRef<str>,
        message_thread_id: Option<TelegramMessageThreadId>,
    ) -> String {
        let chat_id = chat_id.as_ref();
        let user_identity = user_identity.as_ref();
        match self {
            Self::ChatUser => format!("{chat_id}:{user_identity}"),
            Self::ChatOnly => chat_id.to_string(),
            Self::UserOnly => user_identity.to_string(),
            Self::ChatThreadUser => {
                let thread = TelegramMessageThreadId::unwrap_or_default(message_thread_id);
                format!("{chat_id}:{thread}:{user_identity}")
            }
        }
    }
}

impl Display for TelegramSessionPartition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::ChatUser => "chat_user",
            Self::ChatOnly => "chat",
            Self::UserOnly => "user",
            Self::ChatThreadUser => "chat_thread_user",
        };
        write!(f, "{value}")
    }
}

impl FromStr for TelegramSessionPartition {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "chat_user" | "chat-user" | "chatuser" => Ok(Self::ChatUser),
            "chat" | "chat_only" | "chat-only" | "chatonly" => Ok(Self::ChatOnly),
            "user" | "user_only" | "user-only" | "useronly" => Ok(Self::UserOnly),
            "chat_thread_user" | "chat-thread-user" | "chatthreaduser" | "topic_user"
            | "topic-user" | "topicuser" => Ok(Self::ChatThreadUser),
            _ => Err("invalid telegram session partition"),
        }
    }
}
