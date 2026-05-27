//! Owns the Studio router config load surface.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use xiuxian_config_core::load_toml_value_with_imports;
use xiuxian_llm::model_routing::{
    WendaoModelRoutingTomlConfig, wendao_model_routing_config_from_toml_value,
};
use xiuxian_wendao::episteme::EpistemeRegistryEntry;

use super::paths::studio_effective_wendao_toml_path;
use super::sanitize::{
    sanitize_path_like, sanitize_path_list, sanitize_projects, sanitize_repo_projects,
};
#[cfg(any(test, feature = "julia"))]
use super::types::WendaoGraphOntologyReadModelQualityEndpointConfig;
use super::types::WendaoTomlConfig;

const DEFAULT_MARKDOWN_PARSER_PLUGIN_ID: &str = "markdown-parser";

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

#[must_use]
pub(crate) fn load_document_extract_endpoint_from_wendao_toml(
    config_root: &Path,
) -> Option<String> {
    let config_path = studio_effective_wendao_toml_path(config_root);
    load_document_extract_endpoint_from_wendao_toml_path(config_path.as_path())
}

#[must_use]
pub(crate) fn load_document_extract_endpoint_from_wendao_toml_path(
    config_path: &Path,
) -> Option<String> {
    if !config_path.is_file() {
        return None;
    }

    let parsed = load_wendao_toml_config(config_path).ok()?;
    normalize_endpoint(parsed.document_extract.endpoint.as_deref())
}

pub(crate) fn load_model_routing_config_from_wendao_toml(
    config_root: &Path,
) -> Result<Option<WendaoModelRoutingTomlConfig>, String> {
    let config_path = studio_effective_wendao_toml_path(config_root);
    load_model_routing_config_from_wendao_toml_path(config_path.as_path())
}

pub(crate) fn load_model_routing_config_from_wendao_toml_path(
    config_path: &Path,
) -> Result<Option<WendaoModelRoutingTomlConfig>, String> {
    if !config_path.is_file() {
        return Ok(None);
    }

    let merged = load_toml_value_with_imports(config_path).map_err(|error| {
        format!(
            "failed to load merged TOML `{}`: {error}",
            config_path.display()
        )
    })?;
    wendao_model_routing_config_from_toml_value(merged).map(Some)
}

pub(crate) fn load_episteme_registry_from_wendao_toml(
    config_root: &Path,
) -> Result<Vec<EpistemeRegistryEntry>, String> {
    let config_path = studio_effective_wendao_toml_path(config_root);
    load_episteme_registry_from_wendao_toml_path(config_path.as_path())
}

pub(crate) fn load_episteme_registry_from_wendao_toml_path(
    config_path: &Path,
) -> Result<Vec<EpistemeRegistryEntry>, String> {
    if !config_path.is_file() {
        return Ok(Vec::new());
    }

    let parsed = load_wendao_toml_config(config_path)?;
    Ok(episteme_registry_entries_from_wendao_toml(parsed))
}

#[must_use]
#[cfg(any(test, feature = "julia"))]
pub(crate) fn load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml(
    config_root: &Path,
) -> Option<WendaoGraphOntologyReadModelQualityEndpointConfig> {
    let config_path = studio_effective_wendao_toml_path(config_root);
    load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml_path(
        config_path.as_path(),
    )
}

#[must_use]
#[cfg(any(test, feature = "julia"))]
pub(crate) fn load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml_path(
    config_path: &Path,
) -> Option<WendaoGraphOntologyReadModelQualityEndpointConfig> {
    if !config_path.is_file() {
        return None;
    }

    let parsed = load_wendao_toml_config(config_path).ok()?;
    let config = parsed.wendaograph.ontology_read_model_quality;
    let base_url = normalize_endpoint(config.base_url.as_deref())?;
    Some(WendaoGraphOntologyReadModelQualityEndpointConfig {
        base_url,
        timeout_seconds: config.timeout_seconds.filter(|value| *value > 0),
        max_in_flight_requests: config.max_in_flight_requests.filter(|value| *value > 0),
    })
}

fn ui_config_from_wendao_toml(parsed: WendaoTomlConfig) -> UiConfig {
    let mut projects = Vec::new();
    let mut repo_projects = Vec::new();
    let global_include_dirs = sanitize_path_list(&parsed.link_graph.include_dirs);
    let mut global_include_dirs_applied = false;

    for (id, project) in parsed.link_graph.projects {
        let root = project
            .root
            .as_deref()
            .and_then(sanitize_path_like)
            .unwrap_or_else(|| ".".to_string());
        let mut dirs = sanitize_path_list(&project.dirs);
        if id == "main" && root == "." && !global_include_dirs.is_empty() {
            dirs = merged_path_list(global_include_dirs.as_slice(), dirs.as_slice());
            global_include_dirs_applied = true;
        }
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
        let plugins = repo_project_plugins_with_defaults(plugins, &mut plugin_seen);

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

    if !global_include_dirs.is_empty() && !global_include_dirs_applied {
        projects.push(UiProjectConfig {
            name: "main".to_string(),
            root: ".".to_string(),
            dirs: global_include_dirs,
        });
    }

    UiConfig {
        projects: sanitize_projects(projects),
        repo_projects: sanitize_repo_projects(repo_projects),
    }
}

fn episteme_registry_entries_from_wendao_toml(
    parsed: WendaoTomlConfig,
) -> Vec<EpistemeRegistryEntry> {
    parsed
        .episteme
        .registries
        .into_iter()
        .map(|(id, entry)| {
            let mut registry_entry = EpistemeRegistryEntry {
                id,
                path: normalized_optional_path(entry.path.as_deref()),
                url: normalized_optional_string(entry.url.as_deref()),
                enabled: entry.enabled.unwrap_or(true),
                subdir: normalized_optional_string(entry.subdir.as_deref())
                    .map_or_else(|| PathBuf::from("."), PathBuf::from),
            };
            if registry_entry.subdir.as_os_str().is_empty() {
                registry_entry.subdir = PathBuf::from(".");
            }
            registry_entry
        })
        .collect()
}

fn normalized_optional_path(raw: Option<&str>) -> Option<PathBuf> {
    normalized_optional_string(raw).map(PathBuf::from)
}

fn normalized_optional_string(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn merged_path_list(left: &[String], right: &[String]) -> Vec<String> {
    let mut paths = left.to_vec();
    paths.extend(right.iter().cloned());
    sanitize_path_list(&paths)
}

fn repo_project_plugins_with_defaults(
    mut plugins: Vec<String>,
    plugin_seen: &mut HashSet<String>,
) -> Vec<String> {
    if plugin_seen.insert(DEFAULT_MARKDOWN_PARSER_PLUGIN_ID.to_string()) {
        plugins.push(DEFAULT_MARKDOWN_PARSER_PLUGIN_ID.to_string());
    }
    plugins
}

fn normalize_endpoint(raw: Option<&str>) -> Option<String> {
    let endpoint = raw?.trim().trim_end_matches('/');
    (!endpoint.is_empty()).then(|| endpoint.to_string())
}
