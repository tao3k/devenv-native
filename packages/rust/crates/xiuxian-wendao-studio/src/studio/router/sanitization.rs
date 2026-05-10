//! Input sanitization utilities for Studio API.

use std::collections::HashSet;

use crate::studio::pathing;
use crate::studio::types::{UiProjectConfig, UiRepoProjectConfig};

/// Sanitizes a list of project configurations.
///
/// Removes duplicates, empty names, and invalid paths.
#[must_use]
pub fn sanitize_projects(raw: Vec<UiProjectConfig>) -> Vec<UiProjectConfig> {
    let mut seen = HashSet::<String>::new();
    let mut out = Vec::new();
    for project in raw {
        let name = project.name.trim();
        if name.is_empty() {
            continue;
        }
        if !seen.insert(name.to_string()) {
            continue;
        }

        let Some(root) = sanitize_path_like(project.root.as_str()) else {
            continue;
        };

        out.push(UiProjectConfig {
            name: name.to_string(),
            root,
            dirs: sanitize_path_list(project.dirs),
        });
    }
    out
}

/// Sanitizes a list of path strings.
///
/// Normalizes paths and removes duplicates.
#[must_use]
pub fn sanitize_path_list(raw: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    raw.into_iter()
        .filter_map(|path| pathing::normalize_project_dir_root(path.as_str()))
        .filter(|normalized| seen.insert(normalized.clone()))
        .collect()
}

/// Sanitizes a list of repo project configurations.
///
/// Validates required fields, removes duplicates, and normalizes paths.
#[must_use]
pub fn sanitize_repo_projects(raw: Vec<UiRepoProjectConfig>) -> Vec<UiRepoProjectConfig> {
    let mut seen = HashSet::<String>::new();
    let mut out = Vec::new();
    for project in raw {
        let id = project.id.trim();
        if id.is_empty() || !seen.insert(id.to_string()) {
            continue;
        }
        let root = project
            .root
            .as_deref()
            .and_then(sanitize_path_like)
            .filter(|value| !value.is_empty());
        let url = project
            .url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let git_ref = project
            .git_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let refresh = project
            .refresh
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let mut plugin_seen = HashSet::<String>::new();
        let plugins = project
            .plugins
            .into_iter()
            .map(|plugin| plugin.trim().to_string())
            .filter(|plugin| !plugin.is_empty())
            .filter(|plugin| plugin_seen.insert(plugin.clone()))
            .collect::<Vec<_>>();
        if plugins.is_empty() {
            continue;
        }
        if root.is_none() && url.is_none() {
            continue;
        }
        out.push(UiRepoProjectConfig {
            id: id.to_string(),
            root,
            url,
            git_ref,
            refresh,
            plugins,
        });
    }
    out
}

/// Sanitizes a single path-like string.
#[must_use]
pub fn sanitize_path_like(raw: &str) -> Option<String> {
    pathing::normalize_path_like(raw)
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/router/sanitization.rs"]
mod tests;
