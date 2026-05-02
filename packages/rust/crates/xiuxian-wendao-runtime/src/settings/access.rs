//! Settings access helpers for YAML-backed runtime configuration.

use serde_yaml::Value;

fn setting_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn setting_value_to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::String(text) => match text.trim().to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        },
        Value::Number(number) => number.as_i64().map(|value| value != 0),
        _ => None,
    }
}

fn get_setting_value<'a>(settings: &'a Value, dotted_key: &str) -> Option<&'a Value> {
    let mut cursor = settings;
    for segment in dotted_key.split('.') {
        match cursor {
            Value::Mapping(map) => {
                let key = Value::String(segment.to_string());
                cursor = map.get(&key)?;
            }
            _ => return None,
        }
    }
    Some(cursor)
}

/// Read a scalar setting as a string from a dotted key path.
#[must_use]
pub fn get_setting_string(settings: &Value, dotted_key: &str) -> Option<String> {
    get_setting_value(settings, dotted_key).and_then(setting_value_to_string)
}

/// Read a scalar setting as a boolean from a dotted key path.
#[must_use]
pub fn get_setting_bool(settings: &Value, dotted_key: &str) -> Option<bool> {
    get_setting_value(settings, dotted_key).and_then(setting_value_to_bool)
}

/// Read a scalar or sequence setting as a normalized string list.
#[must_use]
pub fn get_setting_string_list(settings: &Value, dotted_key: &str) -> Vec<String> {
    let Some(value) = get_setting_value(settings, dotted_key) else {
        return Vec::new();
    };
    match value {
        Value::String(single) => {
            let text = single.trim();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![text.to_string()]
            }
        }
        Value::Sequence(items) => items
            .iter()
            .filter_map(setting_value_to_string)
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/settings/access.rs"]
mod tests;
