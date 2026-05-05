//! Repository source specification and revision selection contracts.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Specific revision selection policy for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevisionSelector {
    /// Track a specific branch.
    Branch(String),
    /// Pin to a specific tag.
    Tag(String),
    /// Pin to a specific commit SHA.
    Commit(String),
}

impl RevisionSelector {
    /// Returns the revision selector as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Branch(value) | Self::Tag(value) | Self::Commit(value) => value.as_str(),
        }
    }
}

/// Remote refresh policy for one repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepoRefreshPolicy {
    /// Attempt to fetch upstream updates during ensure operations.
    #[default]
    Fetch,
    /// Only refresh on explicit operator request.
    Manual,
}

/// Repository substrate input contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RepoSpec {
    /// Stable repository identifier.
    pub id: String,
    /// Local repository path, when the source is operator-managed.
    #[serde(default)]
    pub local_path: Option<PathBuf>,
    /// Upstream remote URL, when the source is managed by this crate.
    #[serde(default)]
    pub remote_url: Option<String>,
    /// Revision selection policy.
    #[serde(default)]
    pub revision: Option<RevisionSelector>,
    /// Refresh policy for managed remotes.
    #[serde(default)]
    pub refresh: RepoRefreshPolicy,
}
