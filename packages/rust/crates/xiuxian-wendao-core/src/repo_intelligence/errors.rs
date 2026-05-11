//! Error contracts emitted by repository-intelligence planning and analysis.

use thiserror::Error;

use super::projection::ProjectionPageKind;
use super::records::{RepoIntelligencePluginId, RepoRecordId, RepoRecordPath};

macro_rules! repo_error_string_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

repo_error_string_type!(ProjectedPageId, "Stable projected page identifier.");
repo_error_string_type!(ProjectedGapId, "Stable projected gap identifier.");
repo_error_string_type!(
    ProjectedNodeId,
    "Stable projected page-index node identifier."
);

/// Errors raised by the Repo Intelligence common core.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RepoIntelligenceError {
    /// A plugin with the same identifier has already been registered.
    #[error("repo intelligence plugin `{plugin_id}` is already registered")]
    DuplicatePlugin {
        /// The duplicate plugin identifier.
        plugin_id: RepoIntelligencePluginId,
    },
    /// A plugin required by a repository was not found in the registry.
    #[error("repo intelligence plugin `{plugin_id}` is not registered")]
    MissingPlugin {
        /// The missing plugin identifier.
        plugin_id: RepoIntelligencePluginId,
    },
    /// The requested repository does not participate in repo-intelligence analysis.
    #[error("repo `{repo_id}` does not configure any repo-intelligence plugins")]
    MissingRepoIntelligencePlugins {
        /// The repository identifier.
        repo_id: RepoRecordId,
    },
    /// A required plugin was not configured for the requested repository.
    #[error("repo `{repo_id}` requires plugin `{plugin_id}`")]
    MissingRequiredPlugin {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// The required plugin identifier.
        plugin_id: RepoIntelligencePluginId,
    },
    /// A repository id referenced by a query is not registered.
    #[error("repo intelligence repository `{repo_id}` is not registered")]
    UnknownRepository {
        /// The unknown repository identifier.
        repo_id: RepoRecordId,
    },
    /// The repository did not declare a local checkout path.
    #[error("repo `{repo_id}` does not declare a local path")]
    MissingRepositoryPath {
        /// The repository identifier.
        repo_id: RepoRecordId,
    },
    /// The repository did not declare any supported source.
    #[error("repo `{repo_id}` must declare a local path or upstream url")]
    MissingRepositorySource {
        /// The repository identifier.
        repo_id: RepoRecordId,
    },
    /// The configured local repository path is invalid.
    #[error("repo `{repo_id}` has invalid local path `{path}`: {reason}")]
    InvalidRepositoryPath {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// The invalid path.
        path: RepoRecordPath,
        /// Human-readable validation detail.
        reason: String,
    },
    /// The current analyzer does not support the repository layout.
    #[error("repo `{repo_id}` has unsupported layout: {message}")]
    UnsupportedRepositoryLayout {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// Human-readable unsupported-layout detail.
        message: String,
    },
    /// The repository index has not been materialized yet.
    #[error("repo `{repo_id}` index is not ready yet")]
    PendingRepositoryIndex {
        /// The repository identifier.
        repo_id: RepoRecordId,
    },
    /// A deterministic projected page identifier was not found for the repository.
    #[error("repo `{repo_id}` does not contain projected page `{page_id}`")]
    UnknownProjectedPage {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// The missing projected page identifier.
        page_id: ProjectedPageId,
    },
    /// A deterministic projected gap identifier was not found for the repository.
    #[error("repo `{repo_id}` does not contain projected gap `{gap_id}`")]
    UnknownProjectedGap {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// The missing projected gap identifier.
        gap_id: ProjectedGapId,
    },
    /// A deterministic projected page-family cluster was not found for the repository.
    #[error(
        "repo `{repo_id}` does not contain projected page family `{kind:?}` in page `{page_id}`"
    )]
    UnknownProjectedPageFamilyCluster {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// The owning projected page identifier.
        page_id: ProjectedPageId,
        /// The missing projected page family.
        kind: ProjectionPageKind,
    },
    /// A deterministic projected page-index node identifier was not found for the repository.
    #[error(
        "repo `{repo_id}` does not contain projected page-index node `{node_id}` in page `{page_id}`"
    )]
    UnknownProjectedPageIndexNode {
        /// The repository identifier.
        repo_id: RepoRecordId,
        /// The owning projected page identifier.
        page_id: ProjectedPageId,
        /// The missing projected page-index node identifier.
        node_id: ProjectedNodeId,
    },
    /// Configuration loading or parsing failed.
    #[error("repo intelligence config load failed: {message}")]
    ConfigLoad {
        /// Human-readable configuration error detail.
        message: String,
    },
    /// Analysis failed while processing a file or repository.
    #[error("repo intelligence analysis failed: {message}")]
    AnalysisFailed {
        /// Human-readable error detail.
        message: String,
    },
}
