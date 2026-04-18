use std::fs;
use std::path::Path;

use anyhow::Context;
use toml::{Table, Value};

use crate::config::{RuntimeSettings, load_runtime_settings, runtime_settings_paths};
use crate::env_parse::lookup_env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionPartitionPersistenceTarget {
    Telegram,
    Discord,
}

impl SessionPartitionPersistenceTarget {
    const fn section_key(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Discord => "discord",
        }
    }

    const fn persist_env_key(self) -> &'static str {
        match self {
            Self::Telegram => "XIUXIAN_DAOCHANG_TELEGRAM_SESSION_PARTITION_PERSIST",
            Self::Discord => "XIUXIAN_DAOCHANG_DISCORD_SESSION_PARTITION_PERSIST",
        }
    }

    fn persist_setting_value(self, settings: &RuntimeSettings) -> Option<bool> {
        match self {
            Self::Telegram => settings.telegram.session_partition_persist,
            Self::Discord => settings.discord.session_partition_persist,
        }
    }
}

pub(crate) fn persist_session_partition_mode_if_enabled(
    target: SessionPartitionPersistenceTarget,
    mode: &str,
) -> anyhow::Result<bool> {
    let settings = load_runtime_settings();
    if !resolve_session_partition_persist_enabled(target, &settings, |name| {
        std::env::var(name).ok()
    }) {
        return Ok(false);
    }
    let (_, user_settings_path) = runtime_settings_paths();
    persist_session_partition_mode_to_path(user_settings_path.as_path(), target, mode)?;
    Ok(true)
}

pub(crate) fn resolve_session_partition_persist_enabled<F>(
    target: SessionPartitionPersistenceTarget,
    settings: &RuntimeSettings,
    lookup: F,
) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup_env(&lookup, target.persist_env_key()) {
        if let Some(parsed) = parse_bool(raw.as_str()) {
            return parsed;
        }
        tracing::warn!(
            env_var = target.persist_env_key(),
            value = %raw,
            "invalid session partition persistence env value; using settings/default"
        );
    }
    target.persist_setting_value(settings).unwrap_or(false)
}

pub(crate) fn persist_session_partition_mode_to_path(
    user_settings_path: &Path,
    target: SessionPartitionPersistenceTarget,
    mode: &str,
) -> anyhow::Result<()> {
    let normalized_mode = mode.trim();
    if normalized_mode.is_empty() {
        return Err(anyhow::anyhow!(
            "session partition persistence requires non-empty mode"
        ));
    }

    let mut root = load_settings_toml(user_settings_path)?;
    let Some(root_table) = root.as_table_mut() else {
        return Err(anyhow::anyhow!(
            "invalid user settings toml: root must be a table"
        ));
    };
    let section = ensure_child_table(root_table, target.section_key());
    section.insert(
        "session_partition".to_string(),
        Value::String(normalized_mode.to_string()),
    );

    if let Some(parent) = user_settings_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create user settings parent dir: {}",
                parent.display()
            )
        })?;
    }
    let serialized = toml::to_string_pretty(&root)
        .context("failed to serialize user settings toml for session partition persistence")?;
    fs::write(user_settings_path, serialized).with_context(|| {
        format!(
            "failed to write user settings toml: {}",
            user_settings_path.display()
        )
    })?;
    Ok(())
}

fn ensure_child_table<'a>(parent: &'a mut Table, key: &str) -> &'a mut Table {
    let value = parent
        .entry(key.to_string())
        .or_insert_with(|| Value::Table(Table::new()));
    if !value.is_table() {
        *value = Value::Table(Table::new());
    }
    if let Value::Table(table) = value {
        table
    } else {
        // Guarded above; keep a safe fallback path without panic-style unwrap/expect.
        unreachable!("table value should be initialized");
    }
}

fn load_settings_toml(path: &Path) -> anyhow::Result<Value> {
    if !path.exists() {
        return Ok(Value::Table(Table::new()));
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read user settings toml: {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Value::Table(Table::new()));
    }
    let parsed = toml::from_str::<Value>(&raw)
        .with_context(|| format!("failed to parse user settings toml: {}", path.display()))?;
    Ok(parsed)
}

fn parse_bool(raw: &str) -> Option<bool> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
