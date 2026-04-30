use crate::parsers::markdown::{is_supported_note, normalize_alias};
use std::collections::HashSet;
use std::path::Path;

pub(super) fn normalize_include_dir(path: &str) -> Option<String> {
    let normalized = path
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_lowercase();
    if normalized.is_empty() || normalized == "." {
        return None;
    }
    Some(normalized)
}

fn normalize_excluded_dir(name: &str) -> Option<String> {
    let trimmed = name.trim().trim_matches('/').to_lowercase();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed)
}

pub(super) fn merge_excluded_dirs(
    excluded_dirs: &[String],
    default_excluded_dirs: &[&str],
) -> Vec<String> {
    let mut merged: Vec<String> = default_excluded_dirs
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    merged.extend(excluded_dirs.iter().cloned());
    let mut out: Vec<String> = merged
        .into_iter()
        .filter_map(|name| normalize_excluded_dir(&name))
        .collect();
    out.sort();
    out.dedup();
    out
}

fn relative_path_string(path: &Path, root: &Path) -> Option<String> {
    let Ok(relative) = path.strip_prefix(root) else {
        return None;
    };
    let value = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect::<Vec<String>>()
        .join("/");
    Some(value)
}

fn is_under_any_prefix(path: &str, prefixes: &HashSet<String>) -> bool {
    prefixes
        .iter()
        .any(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
}

fn is_ancestor_of_any_prefix(path: &str, prefixes: &HashSet<String>) -> bool {
    prefixes.iter().any(|prefix| {
        if path.is_empty() {
            return true;
        }
        prefix == path || prefix.starts_with(&format!("{path}/"))
    })
}

pub(super) fn should_skip_entry(
    path: &Path,
    is_dir: bool,
    root: &Path,
    include_dirs: &HashSet<String>,
    excluded_dirs: &HashSet<String>,
) -> bool {
    let Some(relative) = relative_path_string(path, root) else {
        return false;
    };

    if !include_dirs.is_empty()
        && !is_under_any_prefix(&relative, include_dirs)
        && !is_ancestor_of_any_prefix(&relative, include_dirs)
    {
        return true;
    }

    let mut prefix_components = Vec::new();
    let mut components = relative
        .split('/')
        .filter(|value| !value.is_empty())
        .peekable();
    while let Some(component) = components.next() {
        let is_last = components.peek().is_none();
        if !is_dir && is_last {
            break;
        }
        prefix_components.push(component);
        let current_prefix = prefix_components.join("/");
        if component.starts_with('.') {
            let explicitly_included = !include_dirs.is_empty()
                && (is_under_any_prefix(&current_prefix, include_dirs)
                    || is_ancestor_of_any_prefix(&current_prefix, include_dirs));
            if explicitly_included {
                continue;
            }
            return true;
        }
    }

    if excluded_dirs.is_empty() {
        return false;
    }

    let mut prefix_components = Vec::new();
    let mut components = relative
        .split('/')
        .filter(|value| !value.is_empty())
        .peekable();
    while let Some(component) = components.next() {
        let is_last = components.peek().is_none();
        if !is_dir && is_last {
            break;
        }
        prefix_components.push(component);
        let current_prefix = prefix_components.join("/");
        let is_excluded = excluded_dirs.contains(component.to_lowercase().as_str());
        if !is_excluded {
            continue;
        }
        let explicitly_included = !include_dirs.is_empty()
            && (is_under_any_prefix(&current_prefix, include_dirs)
                || is_ancestor_of_any_prefix(&current_prefix, include_dirs));
        if explicitly_included {
            continue;
        }
        return true;
    }

    false
}

pub(super) fn is_supported_note_candidate(path: &Path) -> bool {
    if is_supported_note(path) {
        return true;
    }
    path.extension()
        .and_then(|v| v.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "md" | "markdown" | "mdx" | "org"
            )
        })
}

pub(super) fn normalized_relative_note_alias(path: &Path, root: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let raw = relative.to_string_lossy().replace('\\', "/");
    let normalized = normalize_alias(&raw);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}
