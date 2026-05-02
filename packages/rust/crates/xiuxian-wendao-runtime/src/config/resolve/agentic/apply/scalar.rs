//! Scalar setting resolution helpers for agentic runtime config updates.

use crate::settings::{
    first_non_empty, get_setting_string, parse_bool, parse_positive_f64, parse_positive_u64,
    parse_positive_usize,
};
use serde_yaml::Value;

pub(super) fn resolve_usize(settings: &Value, key: &str, env_key: &str) -> Option<usize> {
    first_non_empty(&[
        get_setting_string(settings, key),
        std::env::var(env_key).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
}

pub(super) fn resolve_u64(settings: &Value, key: &str, env_key: &str) -> Option<u64> {
    first_non_empty(&[
        get_setting_string(settings, key),
        std::env::var(env_key).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_u64)
}

pub(super) fn resolve_f64(settings: &Value, key: &str, env_key: &str) -> Option<f64> {
    first_non_empty(&[
        get_setting_string(settings, key),
        std::env::var(env_key).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_f64)
}

pub(super) fn resolve_bool(settings: &Value, key: &str, env_key: &str) -> Option<bool> {
    first_non_empty(&[
        get_setting_string(settings, key),
        std::env::var(env_key).ok(),
    ])
    .as_deref()
    .and_then(parse_bool)
}

pub(super) fn resolve_non_empty_string(
    settings: &Value,
    key: &str,
    env_key: &str,
) -> Option<String> {
    first_non_empty(&[
        get_setting_string(settings, key),
        std::env::var(env_key).ok(),
    ])
    .and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
