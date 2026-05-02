use std::fs;

use xiuxian_daochang::RuntimeSettings;
use xiuxian_daochang::test_support::{
    SessionPartitionPersistenceTarget, persist_session_partition_mode_to_path,
    resolve_session_partition_persist_enabled,
};

fn require_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn require_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

#[test]
fn session_partition_persist_enabled_uses_settings_when_env_absent() {
    let mut settings = RuntimeSettings::default();
    settings.telegram.session_partition_persist = Some(true);

    let enabled = resolve_session_partition_persist_enabled(
        SessionPartitionPersistenceTarget::Telegram,
        &settings,
        |_| None,
    );
    assert!(enabled);
}

#[test]
fn session_partition_persist_enabled_env_overrides_settings() {
    let mut settings = RuntimeSettings::default();
    settings.discord.session_partition_persist = Some(true);

    let enabled = resolve_session_partition_persist_enabled(
        SessionPartitionPersistenceTarget::Discord,
        &settings,
        |_| Some("false".to_string()),
    );
    assert!(!enabled);
}

#[test]
fn persist_session_partition_mode_writes_telegram_section() {
    let temp_dir = require_ok(tempfile::tempdir(), "create tempdir");
    let settings_path = temp_dir.path().join("xiuxian.toml");
    require_ok(
        fs::write(
            &settings_path,
            r#"
[agent]
llm_backend = "litellm_rs"
"#,
        ),
        "write base settings",
    );

    require_ok(
        persist_session_partition_mode_to_path(
            settings_path.as_path(),
            SessionPartitionPersistenceTarget::Telegram,
            "chat_user",
        ),
        "persist telegram partition mode",
    );

    let raw = require_ok(fs::read_to_string(&settings_path), "read settings");
    let parsed = require_ok(toml::from_str::<toml::Value>(&raw), "parse settings");
    let telegram = require_some(
        parsed.get("telegram").and_then(toml::Value::as_table),
        "telegram section",
    );
    assert_eq!(
        telegram
            .get("session_partition")
            .and_then(toml::Value::as_str),
        Some("chat_user")
    );
}

#[test]
fn persist_session_partition_mode_updates_existing_discord_value() {
    let temp_dir = require_ok(tempfile::tempdir(), "create tempdir");
    let settings_path = temp_dir.path().join("xiuxian.toml");
    require_ok(
        fs::write(
            &settings_path,
            r#"
[discord]
runtime_mode = "gateway"
session_partition = "channel"
"#,
        ),
        "write base settings",
    );

    require_ok(
        persist_session_partition_mode_to_path(
            settings_path.as_path(),
            SessionPartitionPersistenceTarget::Discord,
            "guild_user",
        ),
        "persist discord partition mode",
    );

    let raw = require_ok(fs::read_to_string(&settings_path), "read settings");
    let parsed = require_ok(toml::from_str::<toml::Value>(&raw), "parse settings");
    let discord = require_some(
        parsed.get("discord").and_then(toml::Value::as_table),
        "discord section",
    );
    assert_eq!(
        discord
            .get("session_partition")
            .and_then(toml::Value::as_str),
        Some("guild_user")
    );
    assert_eq!(
        discord.get("runtime_mode").and_then(toml::Value::as_str),
        Some("gateway")
    );
}
