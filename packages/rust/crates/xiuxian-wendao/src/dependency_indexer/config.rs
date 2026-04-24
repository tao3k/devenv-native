//! Dependency Config - Load external dependency settings from TOML config.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// External dependency configuration (renamed to avoid conflict).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    from = "ConfigExternalDependencyHelper",
    into = "ConfigExternalDependencyHelper"
)]
pub struct ConfigExternalDependency {
    /// Package type: "rust" or "python"
    pub pkg_type: String,
    /// Registry type: "cargo" or "pip"
    pub registry: Option<String>,
    /// List of manifest file patterns
    pub manifests: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct ConfigExternalDependencyHelper {
    #[serde(rename = "type")]
    pkg_type: String,
    registry: Option<String>,
    manifests: Vec<String>,
}

impl From<ConfigExternalDependencyHelper> for ConfigExternalDependency {
    fn from(helper: ConfigExternalDependencyHelper) -> Self {
        Self {
            pkg_type: helper.pkg_type,
            registry: helper.registry,
            manifests: helper.manifests,
        }
    }
}

impl From<ConfigExternalDependency> for ConfigExternalDependencyHelper {
    fn from(dep: ConfigExternalDependency) -> Self {
        Self {
            pkg_type: dep.pkg_type,
            registry: dep.registry,
            manifests: dep.manifests,
        }
    }
}

/// Dependency configuration loaded from a TOML config file.
#[derive(Debug, Clone, Default)]
pub struct DependencyConfig {
    /// List of external dependency configurations
    pub manifests: Vec<ConfigExternalDependency>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct DependencyConfigFile {
    #[serde(default)]
    ast_symbols_external: Vec<ConfigExternalDependency>,
}

impl DependencyConfig {
    /// Load configuration from a TOML file.
    #[must_use]
    pub fn load(path: &str) -> Self {
        let path = expand_home_path(path, resolve_home_dir().as_deref());

        if !path.exists() {
            log::warn!("Config file not found: {}", path.display());
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<DependencyConfigFile>(&content) {
                Ok(config) => {
                    let manifests = config
                        .ast_symbols_external
                        .into_iter()
                        .filter(|entry| !entry.pkg_type.is_empty() && !entry.manifests.is_empty())
                        .collect();
                    Self { manifests }
                }
                Err(error) => {
                    log::warn!("Failed to parse config '{}': {error}", path.display());
                    Self::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to read config: {e}");
                Self::default()
            }
        }
    }
}

fn expand_home_path(path: &str, home_dir: Option<&Path>) -> PathBuf {
    if let Some(stripped) = path.strip_prefix('~')
        && let Some(home) = home_dir
    {
        return home.join(stripped.trim_start_matches('/'));
    }

    PathBuf::from(path)
}

fn resolve_home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| {
            let drive = env::var_os("HOMEDRIVE")?;
            let path = env::var_os("HOMEPATH")?;
            let mut combined = PathBuf::from(drive);
            combined.push(path);
            Some(combined)
        })
}

#[cfg(test)]
#[path = "../../tests/unit/dependency_indexer/config.rs"]
mod tests;
