use xiuxian_macros::env_non_empty;

use crate::config::load_runtime_settings;
use crate::env_parse::{
    parse_positive_u64_from_env, parse_positive_usize_from_env, resolve_valkey_url_env,
};

pub(super) const DEFAULT_SESSION_KEY_PREFIX: &str = "xiuxian-daochang:session";

#[derive(Debug, Clone)]
pub(crate) struct RedisSessionConfig {
    pub(crate) url: String,
    pub(crate) key_prefix: String,
    pub(crate) ttl_secs: Option<u64>,
    pub(crate) message_content_max_chars: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RedisSessionRuntimeSnapshot {
    pub(crate) url: String,
    pub(crate) key_prefix: String,
    pub(crate) ttl_secs: Option<u64>,
    pub(crate) message_content_max_chars: Option<usize>,
}

impl RedisSessionConfig {
    pub(crate) fn from_env() -> Option<Self> {
        let settings = load_runtime_settings();
        let url = settings
            .session
            .valkey_url
            .as_deref()
            .map(str::trim)
            .map(str::to_string)
            .filter(|v| !v.is_empty())
            .or_else(resolve_valkey_url_env)?;
        let key_prefix = env_non_empty!("XIUXIAN_DAOCHANG_SESSION_VALKEY_PREFIX")
            .or_else(|| {
                settings
                    .session
                    .redis_prefix
                    .as_deref()
                    .map(str::trim)
                    .map(str::to_string)
                    .filter(|v| !v.is_empty())
            })
            .unwrap_or_else(|| DEFAULT_SESSION_KEY_PREFIX.to_string());
        let ttl_secs = parse_positive_u64_from_env("XIUXIAN_DAOCHANG_SESSION_TTL_SECS")
            .or(settings.session.ttl_secs.filter(|v| *v > 0));
        let message_content_max_chars = parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_SESSION_MESSAGE_CONTENT_MAX_CHARS",
        )
        .or(settings
            .session
            .message_content_max_chars
            .filter(|value| *value > 0));
        Some(Self {
            url,
            key_prefix,
            ttl_secs,
            message_content_max_chars,
        })
    }
}
