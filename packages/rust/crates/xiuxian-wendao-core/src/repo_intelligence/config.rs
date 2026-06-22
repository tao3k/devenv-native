//! Configuration records for repository-intelligence plugin execution.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Full configuration for the Repo Intelligence runtime.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoIntelligenceConfig {
    /// List of registered repositories available for analysis.
    pub repos: Vec<RegisteredRepository>,
}

/// One repository registered with the Repo Intelligence runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RegisteredRepository {
    /// Stable repository identifier used by CLI, gateway, and query APIs.
    pub id: String,
    /// Local repository checkout path used for the current MVP slice.
    #[serde(default)]
    pub path: Option<PathBuf>,
    /// Upstream git URL.
    #[serde(default)]
    pub url: Option<String>,
    /// Revision policy to materialize locally.
    #[serde(rename = "ref", default)]
    pub git_ref: Option<RepositoryRef>,
    /// Refresh policy for upstream updates.
    #[serde(default)]
    pub refresh: RepositoryRefreshPolicy,
    /// Analysis plugins associated with the repository.
    #[serde(default)]
    pub plugins: Vec<RepositoryPluginConfig>,
}

impl RegisteredRepository {
    /// Returns all configured plugin identifiers in sorted order.
    #[must_use]
    pub fn configured_plugin_ids(&self) -> Vec<String> {
        let mut plugin_ids = self
            .plugins
            .iter()
            .map(|plugin| plugin.id().to_string())
            .collect::<Vec<_>>();
        plugin_ids.sort_unstable();
        plugin_ids.dedup();
        plugin_ids
    }

    /// Returns the configured repo-intelligence plugins for this repository.
    pub fn repo_intelligence_plugins(&self) -> impl Iterator<Item = &RepositoryPluginConfig> + '_ {
        self.plugins
            .iter()
            .filter(|plugin| plugin.is_repo_intelligence_plugin())
    }

    /// Returns whether the repository has any repo-intelligence plugins.
    #[must_use]
    pub fn has_repo_intelligence_plugins(&self) -> bool {
        self.repo_intelligence_plugins().next().is_some()
    }

    /// Returns the stable repo-intelligence plugin identifiers in sorted order.
    #[must_use]
    pub fn repo_intelligence_plugin_ids(&self) -> Vec<String> {
        self.repo_intelligence_plugins()
            .map(|plugin| plugin.id().to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

/// Specific git reference to materialize.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryRef {
    /// Track a specific branch.
    Branch(String),
    /// Pin to a specific tag.
    Tag(String),
    /// Pin to a specific commit SHA.
    Commit(String),
}

impl RepositoryRef {
    /// Returns the string representation of the reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Branch(s) | Self::Tag(s) | Self::Commit(s) => s.as_str(),
        }
    }
}

/// Policy for repository source refreshing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryRefreshPolicy {
    /// Always attempt to fetch upstream updates on every analysis.
    #[default]
    Fetch,
    /// Only perform manual source refreshes.
    Manual,
}

/// Configuration for a repository analysis plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryPluginConfig {
    /// Plugin identified by its stable ID.
    Id(String),
    /// Inline plugin configuration.
    Config {
        /// Plugin identifier.
        id: String,
        /// Plugin-specific options.
        options: serde_json::Value,
    },
}

impl RepositoryPluginConfig {
    /// Returns the stable plugin identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Id(id) | Self::Config { id, .. } => id.as_str(),
        }
    }

    /// Returns whether this plugin participates in repo-intelligence analysis.
    #[must_use]
    pub fn is_repo_intelligence_plugin(&self) -> bool {
        !self.is_search_only_plugin()
    }

    /// Returns whether this plugin is search-only and should not enter repo intelligence.
    #[must_use]
    pub fn is_search_only_plugin(&self) -> bool {
        matches!(self.id(), "repo-content" | "ast-grep")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/repo_intelligence/config.rs"]
mod tests;
