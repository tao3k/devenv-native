use crate::{DiscordRuntimeMode, TelegramChannelMode, WebhookDedupBackendMode};
use xiuxian_macros::env_non_empty;

pub(crate) fn resolve_string(
    cli_value: Option<String>,
    env_name: &str,
    settings_value: Option<&str>,
    default: &str,
) -> String {
    if let Some(value) = cli_value {
        return value;
    }
    if let Some(value) = read_non_empty_env(env_name) {
        return value;
    }
    if let Some(value) = settings_value {
        return value.to_string();
    }
    default.to_string()
}

pub(crate) fn resolve_positive_u64(
    cli_value: Option<u64>,
    env_name: &str,
    settings_value: Option<u64>,
    default: u64,
) -> u64 {
    if let Some(value) = cli_value
        && value > 0
    {
        return value;
    }
    if let Some(value) = parse_positive_u64_from_env(env_name) {
        return value;
    }
    if let Some(value) = settings_value
        && value > 0
    {
        return value;
    }
    default
}

pub(crate) fn resolve_positive_usize(
    cli_value: Option<usize>,
    env_name: &str,
    settings_value: Option<usize>,
    default: usize,
) -> usize {
    if let Some(value) = cli_value
        && value > 0
    {
        return value;
    }
    if let Some(value) = parse_positive_usize_from_env(env_name) {
        return value;
    }
    if let Some(value) = settings_value
        && value > 0
    {
        return value;
    }
    default
}

pub(crate) fn resolve_bool(
    cli_value: Option<bool>,
    env_name: &str,
    settings_value: Option<bool>,
    default: bool,
) -> bool {
    if let Some(value) = cli_value {
        return value;
    }
    if let Some(value) = parse_bool_from_env(env_name) {
        return value;
    }
    if let Some(value) = settings_value {
        return value;
    }
    default
}

pub(crate) fn resolve_channel_mode(
    cli_mode: Option<TelegramChannelMode>,
    settings_mode: Option<&str>,
) -> TelegramChannelMode {
    if let Some(mode) = cli_mode {
        return mode;
    }
    if let Some(raw) = read_non_empty_env("XIUXIAN_DAOCHANG_TELEGRAM_MODE") {
        if let Some(mode) = parse_channel_mode(&raw) {
            return mode;
        }
        tracing::warn!(
            env_var = "XIUXIAN_DAOCHANG_TELEGRAM_MODE",
            value = %raw,
            "invalid telegram mode env value; using settings/default"
        );
    }
    if let Some(raw) = settings_mode {
        if let Some(mode) = parse_channel_mode(raw) {
            return mode;
        }
        tracing::warn!(
            value = %raw,
            "invalid telegram.mode in settings; using default"
        );
    }
    TelegramChannelMode::Webhook
}

pub(crate) fn resolve_dedup_backend(
    cli_backend: Option<WebhookDedupBackendMode>,
    settings_backend: Option<&str>,
) -> WebhookDedupBackendMode {
    if let Some(backend) = cli_backend {
        return backend;
    }
    if let Some(raw) = read_non_empty_env("XIUXIAN_DAOCHANG_TELEGRAM_WEBHOOK_DEDUP_BACKEND") {
        if let Some(backend) = parse_dedup_backend(&raw) {
            return backend;
        }
        tracing::warn!(
            env_var = "XIUXIAN_DAOCHANG_TELEGRAM_WEBHOOK_DEDUP_BACKEND",
            value = %raw,
            "invalid telegram webhook dedup backend env value; using settings/default"
        );
    }
    if let Some(raw) = settings_backend {
        if let Some(backend) = parse_dedup_backend(raw) {
            return backend;
        }
        tracing::warn!(
            value = %raw,
            "invalid telegram.webhook_dedup_backend in settings; using default"
        );
    }
    WebhookDedupBackendMode::Valkey
}

pub(crate) fn resolve_discord_runtime_mode(
    cli_mode: Option<DiscordRuntimeMode>,
    settings_mode: Option<&str>,
) -> DiscordRuntimeMode {
    if let Some(mode) = cli_mode {
        return mode;
    }
    if let Some(raw) = read_non_empty_env("XIUXIAN_DAOCHANG_DISCORD_RUNTIME_MODE") {
        if let Some(mode) = parse_discord_runtime_mode(&raw) {
            return mode;
        }
        tracing::warn!(
            env_var = "XIUXIAN_DAOCHANG_DISCORD_RUNTIME_MODE",
            value = %raw,
            "invalid discord runtime mode env value; using settings/default"
        );
    }
    if let Some(raw) = settings_mode {
        if let Some(mode) = parse_discord_runtime_mode(raw) {
            return mode;
        }
        tracing::warn!(
            value = %raw,
            "invalid discord.runtime_mode in settings; using default"
        );
    }
    DiscordRuntimeMode::Gateway
}

fn parse_channel_mode(raw: &str) -> Option<TelegramChannelMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "polling" => Some(TelegramChannelMode::Polling),
        "webhook" => Some(TelegramChannelMode::Webhook),
        _ => None,
    }
}

fn parse_discord_runtime_mode(raw: &str) -> Option<DiscordRuntimeMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "gateway" => Some(DiscordRuntimeMode::Gateway),
        "ingress" => Some(DiscordRuntimeMode::Ingress),
        _ => None,
    }
}

fn parse_dedup_backend(raw: &str) -> Option<WebhookDedupBackendMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "memory" => Some(WebhookDedupBackendMode::Memory),
        "valkey" | "redis" => Some(WebhookDedupBackendMode::Valkey),
        _ => None,
    }
}

fn trim_non_empty(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[must_use]
pub(crate) fn resolve_valkey_url_env() -> Option<String> {
    std::env::var("XIUXIAN_WENDAO_VALKEY_URL")
        .ok()
        .as_deref()
        .and_then(trim_non_empty)
        .or_else(|| {
            std::env::var("VALKEY_URL")
                .ok()
                .as_deref()
                .and_then(trim_non_empty)
        })
}

#[must_use]
pub(crate) fn parse_positive_usize_from_env(name: &str) -> Option<usize> {
    parse_env_value(
        name,
        |raw| raw.parse::<usize>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

#[must_use]
pub(crate) fn parse_positive_u64_from_env(name: &str) -> Option<u64> {
    parse_env_value(
        name,
        |raw| raw.parse::<u64>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

fn parse_env_value<T>(
    name: &str,
    parser: impl FnOnce(&str) -> Option<T>,
    invalid_message: &'static str,
) -> Option<T> {
    let raw = read_env(name)?;
    if let Some(value) = parser(raw.as_str()) {
        Some(value)
    } else {
        tracing::warn!(env_var = %name, value = %raw, "{invalid_message}");
        None
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[must_use]
pub(crate) fn parse_bool_from_env(name: &str) -> Option<bool> {
    parse_env_value(name, parse_bool, "invalid bool env value")
}

fn read_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn read_non_empty_env(name: &str) -> Option<String> {
    env_non_empty!(name)
}
