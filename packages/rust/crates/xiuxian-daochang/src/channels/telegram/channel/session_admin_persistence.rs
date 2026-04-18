use std::fs;
use std::path::Path;

use anyhow::Context;
use toml::{Value, map::Map};

pub(super) fn persist_session_admin_override_to_user_settings(
    user_settings_path: &Path,
    recipient: &str,
    admin_users: Option<&[String]>,
) -> anyhow::Result<()> {
    let scope = parse_scope(recipient)?;
    let mut root = load_settings_toml(user_settings_path)?;
    let Some(root_map) = root.as_table_mut() else {
        return Err(anyhow::anyhow!(
            "invalid user settings toml: root must be a table"
        ));
    };

    let changed = match scope {
        SessionAdminScope::Group { chat_id } => {
            apply_group_admin_override(root_map, chat_id.as_str(), admin_users)
        }
        SessionAdminScope::Topic { chat_id, thread_id } => {
            apply_topic_admin_override(root_map, chat_id.as_str(), thread_id, admin_users)
        }
    };
    if !changed {
        return Ok(());
    }

    if let Some(parent) = user_settings_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create user settings parent dir: {}",
                parent.display()
            )
        })?;
    }
    let serialized = toml::to_string(&root)
        .context("failed to serialize user settings toml for session admin persistence")?;
    fs::write(user_settings_path, serialized).with_context(|| {
        format!(
            "failed to write user settings toml: {}",
            user_settings_path.display()
        )
    })?;
    Ok(())
}

enum SessionAdminScope {
    Group { chat_id: String },
    Topic { chat_id: String, thread_id: i64 },
}

fn parse_scope(recipient: &str) -> anyhow::Result<SessionAdminScope> {
    let (chat_id, thread_id) = super::identity::parse_recipient_target(recipient);
    if !chat_id.starts_with('-') {
        return Err(anyhow::anyhow!(
            "recipient-scoped admin override is only supported for group chats"
        ));
    }
    match thread_id {
        Some(raw_thread_id) => {
            let parsed = raw_thread_id
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("invalid topic id in recipient: {recipient}"))?;
            if parsed <= 0 {
                return Err(anyhow::anyhow!(
                    "invalid topic id in recipient: {recipient}"
                ));
            }
            Ok(SessionAdminScope::Topic {
                chat_id: chat_id.to_string(),
                thread_id: parsed,
            })
        }
        None => Ok(SessionAdminScope::Group {
            chat_id: chat_id.to_string(),
        }),
    }
}

fn load_settings_toml(path: &Path) -> anyhow::Result<Value> {
    if !path.exists() {
        return Ok(Value::Table(Map::new()));
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read user settings toml: {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Value::Table(Map::new()));
    }
    let parsed = toml::from_str::<Value>(&raw)
        .with_context(|| format!("failed to parse user settings toml: {}", path.display()))?;
    Ok(match parsed {
        Value::String(_)
        | Value::Integer(_)
        | Value::Float(_)
        | Value::Boolean(_)
        | Value::Datetime(_)
        | Value::Array(_) => Value::Table(Map::new()),
        other @ Value::Table(_) => other,
    })
}

fn apply_group_admin_override(
    root_map: &mut Map<String, Value>,
    chat_id: &str,
    admin_users: Option<&[String]>,
) -> bool {
    let Some(telegram_map) = ensure_child_table(root_map, "telegram", admin_users.is_some()) else {
        return false;
    };
    let Some(groups_by_chat) = ensure_child_table(telegram_map, "groups", admin_users.is_some())
    else {
        return false;
    };

    if let Some(entries) = admin_users {
        let group_value = groups_by_chat
            .entry(chat_id.to_string())
            .or_insert_with(|| Value::Table(Map::new()));
        let Some(group_entry_map) = ensure_value_table(group_value) else {
            return false;
        };
        set_admin_users(group_entry_map, entries);
        true
    } else {
        let Some(group_value) = groups_by_chat.get_mut(chat_id) else {
            return false;
        };
        let Some(group_entry_map) = ensure_value_table(group_value) else {
            return false;
        };
        let changed = group_entry_map.remove("admin_users").is_some();
        if changed && group_entry_map.is_empty() {
            groups_by_chat.remove(chat_id);
        }
        prune_empty_groups_and_telegram(root_map);
        changed
    }
}

fn apply_topic_admin_override(
    root_map: &mut Map<String, Value>,
    chat_id: &str,
    thread_id: i64,
    admin_users: Option<&[String]>,
) -> bool {
    let Some(telegram_map) = ensure_child_table(root_map, "telegram", admin_users.is_some()) else {
        return false;
    };
    let Some(groups_by_chat) = ensure_child_table(telegram_map, "groups", admin_users.is_some())
    else {
        return false;
    };
    let topic_key = thread_id.to_string();

    if let Some(entries) = admin_users {
        let group_value = groups_by_chat
            .entry(chat_id.to_string())
            .or_insert_with(|| Value::Table(Map::new()));
        let Some(group_entry_map) = ensure_value_table(group_value) else {
            return false;
        };
        let Some(topics_by_thread) = ensure_child_table(group_entry_map, "topics", true) else {
            return false;
        };
        let topic_value = topics_by_thread
            .entry(topic_key.clone())
            .or_insert_with(|| Value::Table(Map::new()));
        let Some(topic_entry_map) = ensure_value_table(topic_value) else {
            return false;
        };
        set_admin_users(topic_entry_map, entries);
        true
    } else {
        let Some(group_value) = groups_by_chat.get_mut(chat_id) else {
            return false;
        };
        let Some(group_entry_map) = ensure_value_table(group_value) else {
            return false;
        };
        let Some(topics_node) = group_entry_map.get_mut("topics") else {
            return false;
        };
        let Some(topics_by_thread) = ensure_value_table(topics_node) else {
            return false;
        };
        let Some(topic_entry) = topics_by_thread.get_mut(&topic_key) else {
            return false;
        };
        let Some(topic_entry_map) = ensure_value_table(topic_entry) else {
            return false;
        };
        let changed = topic_entry_map.remove("admin_users").is_some();
        if changed && topic_entry_map.is_empty() {
            topics_by_thread.remove(&topic_key);
        }
        if topics_by_thread.is_empty() {
            group_entry_map.remove("topics");
        }
        if group_entry_map.is_empty() {
            groups_by_chat.remove(chat_id);
        }
        prune_empty_groups_and_telegram(root_map);
        changed
    }
}

fn ensure_child_table<'a>(
    parent: &'a mut Map<String, Value>,
    key: &str,
    create_if_missing: bool,
) -> Option<&'a mut Map<String, Value>> {
    if !parent.contains_key(key) {
        if !create_if_missing {
            return None;
        }
        parent.insert(key.to_string(), Value::Table(Map::new()));
    }
    let value = parent.get_mut(key)?;
    ensure_value_table(value)
}

fn ensure_value_table(value: &mut Value) -> Option<&mut Map<String, Value>> {
    if !matches!(value, Value::Table(_)) {
        return None;
    }
    value.as_table_mut()
}

fn set_admin_users(target: &mut Map<String, Value>, admin_users: &[String]) {
    let users = admin_users
        .iter()
        .map(|entry| Value::String(entry.clone()))
        .collect::<Vec<_>>();
    let mut principal_map = Map::new();
    principal_map.insert("users".to_string(), Value::Array(users));
    target.insert("admin_users".to_string(), Value::Table(principal_map));
}

fn prune_empty_groups_and_telegram(root_map: &mut Map<String, Value>) {
    let Some(telegram_value) = root_map.get_mut("telegram") else {
        return;
    };
    let Some(telegram_map) = ensure_value_table(telegram_value) else {
        return;
    };
    let remove_groups = telegram_map
        .get("groups")
        .and_then(Value::as_table)
        .is_some_and(Map::is_empty);
    if remove_groups {
        telegram_map.remove("groups");
    }
    if telegram_map.is_empty() {
        root_map.remove("telegram");
    }
}
