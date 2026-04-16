use std::path::{Component, Path, PathBuf};

use crate::analyzers::RepoIntelligenceError;

use super::toml::{WendaoTomlPluginEntry, WendaoTomlPluginInlineConfig};
use super::types::{RepositoryPluginConfig, RepositoryRef, RepositoryRefreshPolicy};

pub(crate) fn parse_repository_plugins(
    plugins: Vec<WendaoTomlPluginEntry>,
    repo_id: &str,
    config_path: &Path,
) -> Result<Vec<RepositoryPluginConfig>, RepoIntelligenceError> {
    plugins
        .into_iter()
        .filter_map(|plugin| match plugin {
            WendaoTomlPluginEntry::Id(id) => {
                let id = id.trim().to_string();
                if id.is_empty() {
                    None
                } else {
                    Some(Ok(RepositoryPluginConfig::Id(id)))
                }
            }
            WendaoTomlPluginEntry::Config(config) => {
                Some(parse_inline_plugin_config(config, repo_id, config_path))
            }
        })
        .collect()
}

pub(crate) fn parse_inline_plugin_config(
    plugin: WendaoTomlPluginInlineConfig,
    repo_id: &str,
    config_path: &Path,
) -> Result<RepositoryPluginConfig, RepoIntelligenceError> {
    let id = plugin.id.trim();
    if id.is_empty() {
        return Err(RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to parse `{}`: repo `{repo_id}` plugin id cannot be empty",
                config_path.display()
            ),
        });
    }

    let options =
        serde_json::to_value(plugin.options).map_err(|error| RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to parse `{}`: repo `{repo_id}` plugin `{id}` options cannot be encoded as JSON: {error}",
                config_path.display()
            ),
        })?;

    Ok(RepositoryPluginConfig::Config {
        id: id.to_string(),
        options,
    })
}

pub(crate) fn parse_repository_ref(value: &str) -> Option<RepositoryRef> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(reference) = parse_prefixed_repository_ref(trimmed, "branch:") {
        return Some(RepositoryRef::Branch(reference));
    }
    if let Some(reference) = parse_prefixed_repository_ref(trimmed, "tag:") {
        return Some(RepositoryRef::Tag(reference));
    }
    if let Some(reference) = parse_prefixed_repository_ref(trimmed, "commit:") {
        return Some(RepositoryRef::Commit(reference));
    }

    Some(RepositoryRef::Branch(trimmed.to_string()))
}

fn parse_prefixed_repository_ref(value: &str, prefix: &str) -> Option<String> {
    value
        .strip_prefix(prefix)
        .map(str::trim)
        .filter(|reference| !reference.is_empty())
        .map(str::to_string)
}

pub(crate) fn parse_refresh_policy(value: Option<&str>) -> RepositoryRefreshPolicy {
    match value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("fetch")
    {
        "manual" => RepositoryRefreshPolicy::Manual,
        _ => RepositoryRefreshPolicy::Fetch,
    }
}

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let popped = normalized.pop();
                if !popped {
                    normalized.push(component.as_os_str());
                }
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}
