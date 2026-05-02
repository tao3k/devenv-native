//! Discord foreground runtime configuration types.

use crate::channels::discord::session_partition::DiscordSessionPartition;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::config::{DiscordSettings, load_runtime_settings};
use crate::env_parse::lookup_non_empty_env;
use std::collections::HashMap;

const DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY: usize = 512;
const DISCORD_DEFAULT_TURN_TIMEOUT_SECS: u64 = 120;
const DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES: usize = 16;

/// Runtime configuration for Discord ingress/dispatch loop.
#[derive(Debug, Clone)]
pub struct DiscordRuntimeConfig {
    /// Session partition strategy used for Discord messages.
    pub session_partition: DiscordSessionPartition,
    /// Default guild-channel mention gate.
    pub require_mention: bool,
    /// Persist mention-policy runtime mutations.
    pub require_mention_persist: bool,
    /// Per-channel mention overrides keyed by recipient/channel id or `*`.
    pub mention_overrides: HashMap<String, bool>,
    /// Inbound ingress queue capacity.
    pub inbound_queue_capacity: usize,
    /// Per-turn timeout in seconds.
    pub turn_timeout_secs: u64,
    /// Maximum number of in-flight foreground messages.
    pub foreground_max_in_flight_messages: usize,
    /// Foreground queue mode for same-session inbound messages.
    pub foreground_queue_mode: ForegroundQueueMode,
}

impl DiscordRuntimeConfig {
    /// Resolve Discord runtime config from environment-backed defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let settings = load_runtime_settings();
        Self::from_lookup(|name| std::env::var(name).ok(), Some(&settings.discord))
    }

    fn from_lookup<F>(lookup: F, settings: Option<&DiscordSettings>) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let defaults = Self::default();
        Self {
            session_partition: DiscordSessionPartition::from_env(),
            require_mention: resolve_bool(
                &lookup,
                "XIUXIAN_DAOCHANG_DISCORD_REQUIRE_MENTION",
                settings.and_then(|s| s.require_mention),
                defaults.require_mention,
            ),
            require_mention_persist: resolve_bool(
                &lookup,
                "XIUXIAN_DAOCHANG_DISCORD_REQUIRE_MENTION_PERSIST",
                settings.and_then(|s| s.require_mention_persist),
                defaults.require_mention_persist,
            ),
            mention_overrides: settings
                .and_then(|s| s.channels.as_ref())
                .map(|channels| {
                    channels
                        .iter()
                        .filter_map(|(recipient, settings)| {
                            settings.require_mention.map(|require_mention| {
                                (recipient.trim().to_string(), require_mention)
                            })
                        })
                        .filter(|(recipient, _)| !recipient.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            inbound_queue_capacity: resolve_usize(
                &lookup,
                "XIUXIAN_DAOCHANG_DISCORD_INBOUND_QUEUE_CAPACITY",
                settings.and_then(|s| s.inbound_queue_capacity),
                defaults.inbound_queue_capacity,
            ),
            turn_timeout_secs: resolve_u64(
                &lookup,
                "XIUXIAN_DAOCHANG_DISCORD_TURN_TIMEOUT_SECS",
                settings.and_then(|s| s.turn_timeout_secs),
                defaults.turn_timeout_secs,
            ),
            foreground_max_in_flight_messages: resolve_usize(
                &lookup,
                "XIUXIAN_DAOCHANG_DISCORD_FOREGROUND_MAX_IN_FLIGHT_MESSAGES",
                settings.and_then(|s| s.foreground_max_in_flight_messages),
                defaults.foreground_max_in_flight_messages,
            ),
            foreground_queue_mode: resolve_foreground_queue_mode(
                &lookup,
                "XIUXIAN_DAOCHANG_DISCORD_FOREGROUND_QUEUE_MODE",
                settings.and_then(|s| s.foreground_queue_mode.as_deref()),
                defaults.foreground_queue_mode,
            ),
        }
    }
}

impl Default for DiscordRuntimeConfig {
    fn default() -> Self {
        Self {
            session_partition: DiscordSessionPartition::from_env(),
            require_mention: false,
            require_mention_persist: false,
            mention_overrides: HashMap::new(),
            inbound_queue_capacity: DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY,
            turn_timeout_secs: DISCORD_DEFAULT_TURN_TIMEOUT_SECS,
            foreground_max_in_flight_messages: DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES,
            foreground_queue_mode: ForegroundQueueMode::Queue,
        }
    }
}

fn resolve_usize<F>(lookup: &F, name: &str, setting_value: Option<usize>, default: usize) -> usize
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup_non_empty_env(lookup, name) {
        match raw.trim().parse::<usize>() {
            Ok(value) if value > 0 => return value,
            _ => tracing::warn!(
                env_var = %name,
                value = %raw,
                "invalid runtime config env value; using settings/default"
            ),
        }
    }
    match setting_value {
        Some(value) if value > 0 => value,
        Some(value) => {
            tracing::warn!(
                setting = %name,
                value,
                default,
                "invalid runtime config settings value; using default"
            );
            default
        }
        None => default,
    }
}

fn resolve_u64<F>(lookup: &F, name: &str, setting_value: Option<u64>, default: u64) -> u64
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup_non_empty_env(lookup, name) {
        match raw.trim().parse::<u64>() {
            Ok(value) if value > 0 => return value,
            _ => tracing::warn!(
                env_var = %name,
                value = %raw,
                "invalid runtime config env value; using settings/default"
            ),
        }
    }
    match setting_value {
        Some(value) if value > 0 => value,
        Some(value) => {
            tracing::warn!(
                setting = %name,
                value,
                default,
                "invalid runtime config settings value; using default"
            );
            default
        }
        None => default,
    }
}

fn resolve_foreground_queue_mode<F>(
    lookup: &F,
    env_name: &str,
    setting_value: Option<&str>,
    default: ForegroundQueueMode,
) -> ForegroundQueueMode
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup_non_empty_env(lookup, env_name) {
        if let Some(mode) = ForegroundQueueMode::parse(raw.as_str()) {
            return mode;
        }
        tracing::warn!(
            env_var = %env_name,
            value = %raw,
            "invalid queue mode env value; using settings/default"
        );
    }
    if let Some(raw) = setting_value {
        if let Some(mode) = ForegroundQueueMode::parse(raw) {
            return mode;
        }
        tracing::warn!(
            setting = %env_name,
            value = %raw,
            "invalid queue mode settings value; using default"
        );
    }
    default
}

fn resolve_bool<F>(lookup: &F, name: &str, setting_value: Option<bool>, default: bool) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup_non_empty_env(lookup, name) {
        if let Some(value) = parse_bool(raw.as_str()) {
            return value;
        }
        tracing::warn!(
            env_var = %name,
            value = %raw,
            "invalid runtime config bool env value; using settings/default"
        );
    }
    setting_value.unwrap_or(default)
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
