use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use xiuxian_config_core::load_toml_value_with_imports;

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlConfig {
    #[serde(default)]
    sources: WendaoTomlSourcesConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlSourcesConfig {
    #[serde(default)]
    projects: BTreeMap<String, WendaoTomlProjectConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlProjectConfig {
    #[serde(default)]
    root: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    read_only: Option<bool>,
}

pub(super) fn configured_project_roots(root: &Path) -> Result<Vec<PathBuf>> {
    let config_path = root.join("wendao.toml");
    if !config_path.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let merged = load_toml_value_with_imports(config_path.as_path()).with_context(|| {
        format!(
            "failed to load markdown lint config from `{}`",
            config_path.display()
        )
    })?;
    let parsed: WendaoTomlConfig = merged.try_into().with_context(|| {
        format!(
            "failed to parse markdown lint config from `{}`",
            config_path.display()
        )
    })?;
    let config_root = config_path
        .parent()
        .map_or_else(|| root.to_path_buf(), Path::to_path_buf);

    let mut resolved_roots = parsed
        .sources
        .projects
        .into_values()
        .filter(|project| !project_is_read_only(project))
        .filter_map(|project| project.root)
        .map(|configured_root| configured_root.trim().to_string())
        .filter(|configured_root| !configured_root.is_empty())
        .map(|configured_root| resolve_configured_root(config_root.as_path(), &configured_root))
        .collect::<Vec<_>>();

    resolved_roots.sort();
    resolved_roots.dedup();

    if resolved_roots.is_empty() {
        Ok(vec![root.to_path_buf()])
    } else {
        Ok(resolved_roots)
    }
}

fn resolve_configured_root(config_root: &Path, configured_root: &str) -> PathBuf {
    let configured_root = PathBuf::from(configured_root);
    if configured_root.is_absolute() {
        configured_root
    } else {
        config_root.join(configured_root)
    }
}

fn project_is_read_only(project: &WendaoTomlProjectConfig) -> bool {
    project.read_only.unwrap_or(project.url.is_some())
}
