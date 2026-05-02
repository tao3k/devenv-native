//! Hot-reload watch-pattern and extension normalization policy.

use std::collections::HashSet;

/// Resolve normalized watch extension allowlist.
///
/// Precedence:
/// 1) `configured_extensions` (if at least one valid extension remains)
/// 2) `default_extensions`
#[must_use]
pub fn resolve_hot_reload_watch_extensions(
    configured_extensions: Option<&[String]>,
    default_extensions: &[&str],
) -> Vec<String> {
    let configured = normalize_extensions(
        configured_extensions
            .unwrap_or_default()
            .iter()
            .map(String::as_str),
    );
    if !configured.is_empty() {
        return configured;
    }
    normalize_extensions(default_extensions.iter().copied())
}

/// Resolve normalized watch include glob patterns.
///
/// Precedence:
/// 1) `configured_patterns` (if at least one pattern remains)
/// 2) derive from `configured_extensions` as `"**/*.<ext>"`
/// 3) `default_patterns`
#[must_use]
pub fn resolve_hot_reload_watch_patterns(
    configured_patterns: Option<&[String]>,
    configured_extensions: Option<&[String]>,
    default_patterns: &[&str],
) -> Vec<String> {
    let configured = normalize_patterns(
        configured_patterns
            .unwrap_or_default()
            .iter()
            .map(String::as_str),
    );
    if !configured.is_empty() {
        return configured;
    }

    let configured_only_extensions = normalize_extensions(
        configured_extensions
            .unwrap_or_default()
            .iter()
            .map(String::as_str),
    );
    if !configured_only_extensions.is_empty() {
        return configured_only_extensions
            .into_iter()
            .map(|extension| format!("**/*.{extension}"))
            .collect();
    }

    normalize_patterns(default_patterns.iter().copied())
}

fn normalize_patterns<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut unique = HashSet::new();
    let mut normalized = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if unique.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn normalize_extensions<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut unique = HashSet::new();
    let mut normalized = Vec::new();
    for value in values {
        let Some(extension) = normalize_extension(value) else {
            continue;
        };
        if unique.insert(extension.clone()) {
            normalized.push(extension);
        }
    }
    normalized
}

fn normalize_extension(value: &str) -> Option<String> {
    let normalized = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        Some(normalized)
    } else {
        None
    }
}
