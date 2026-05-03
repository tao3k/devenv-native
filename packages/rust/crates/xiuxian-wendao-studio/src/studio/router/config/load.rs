use std::collections::HashSet;
use std::path::Path;

use crate::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use xiuxian_config_core::load_toml_value_with_imports;

use super::paths::studio_effective_wendao_toml_path;
use super::sanitize::{
    sanitize_path_like, sanitize_path_list, sanitize_projects, sanitize_repo_projects,
};
use super::types::WendaoTomlConfig;

/// Loads one merged Wendao TOML config from the effective Studio config path.
///
/// # Errors
///
/// Returns an error string if reading, merging, or deserializing fails.
pub(crate) fn load_wendao_toml_config(path: &Path) -> Result<WendaoTomlConfig, String> {
    let merged = load_toml_value_with_imports(path)
        .map_err(|error| format!("failed to load merged TOML `{}`: {error}", path.display()))?;
    merged.try_into().map_err(|error| {
        format!(
            "failed to deserialize merged TOML `{}`: {error}",
            path.display()
        )
    })
}

/// Loads UI config from the effective Wendao TOML if it exists.
#[must_use]
pub fn load_ui_config_from_wendao_toml(config_root: &Path) -> Option<UiConfig> {
    let config_path = studio_effective_wendao_toml_path(config_root);
    load_ui_config_from_wendao_toml_path(config_path.as_path())
}

/// Loads UI config from one explicit effective Wendao TOML path if it exists.
#[must_use]
pub fn load_ui_config_from_wendao_toml_path(config_path: &Path) -> Option<UiConfig> {
    if !config_path.is_file() {
        return None;
    }

    let parsed = load_wendao_toml_config(config_path).ok()?;
    Some(ui_config_from_wendao_toml(parsed))
}

fn ui_config_from_wendao_toml(parsed: WendaoTomlConfig) -> UiConfig {
    let mut projects = Vec::new();
    let mut repo_projects = Vec::new();

    for (id, project) in parsed.link_graph.projects {
        let dirs = sanitize_path_list(&project.dirs);
        let root = project
            .root
            .as_deref()
            .and_then(sanitize_path_like)
            .unwrap_or_else(|| ".".to_string());
        if !dirs.is_empty() {
            projects.push(UiProjectConfig {
                name: id.clone(),
                root,
                dirs,
            });
        }

        let mut plugin_seen = HashSet::<String>::new();
        let plugins = project
            .plugins
            .into_iter()
            .filter_map(|plugin| plugin.normalized_id())
            .filter(|plugin| plugin_seen.insert(plugin.clone()))
            .collect::<Vec<_>>();
        if plugins.is_empty() {
            continue;
        }

        let repo_root = project.root.as_deref().and_then(sanitize_path_like);
        let url = project
            .url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        if repo_root.is_none() && url.is_none() {
            continue;
        }
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
        repo_projects.push(UiRepoProjectConfig {
            id,
            root: repo_root,
            url,
            git_ref,
            refresh,
            plugins,
        });
    }

    UiConfig {
        projects: sanitize_projects(projects),
        repo_projects: sanitize_repo_projects(repo_projects),
    }
}
