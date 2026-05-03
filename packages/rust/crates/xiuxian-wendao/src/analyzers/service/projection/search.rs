use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(feature = "search-runtime")]
use crate::analyzers::cache::RepositorySearchArtifacts;
use crate::analyzers::projection::build_projected_page_search;
#[cfg(feature = "search-runtime")]
use crate::analyzers::projection::build_repo_projected_page_search_with_artifacts as build_projected_page_search_with_artifacts;
use crate::analyzers::{
    DocsSearchQuery, DocsSearchResult, RepoProjectedPageSearchQuery, RepoProjectedPageSearchResult,
};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build deterministic docs-facing projected-page search results from normalized analysis records.
#[must_use]
pub fn build_docs_search(
    query: &DocsSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsSearchResult {
    build_repo_projected_page_search(
        &RepoProjectedPageSearchQuery {
            repo_id: query.repo_id.clone(),
            query: query.query.clone(),
            kind: query.kind,
            limit: query.limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected-page search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_search_from_config_with_registry(
    query: &DocsSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected-page search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_search_from_config(
    query: &DocsSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_search(query, analysis))
    })
}

/// Build deterministic projected-page search results for one repository query.
#[must_use]
pub fn build_repo_projected_page_search(
    query: &RepoProjectedPageSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPageSearchResult {
    build_projected_page_search(query, analysis)
}

#[must_use]
#[cfg(feature = "search-runtime")]
/// Build deterministic projected-page search results with precomputed artifacts.
pub fn build_repo_projected_page_search_with_artifacts(
    query: &RepoProjectedPageSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    artifacts: &RepositorySearchArtifacts,
) -> RepoProjectedPageSearchResult {
    build_projected_page_search_with_artifacts(query, analysis, artifacts)
}

/// Load configuration, analyze one repository, and return deterministic projected-page search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_page_search_from_config_with_registry(
    query: &RepoProjectedPageSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_repo_projected_page_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic projected-page search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_page_search_from_config(
    query: &RepoProjectedPageSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_repo_projected_page_search(query, analysis))
    })
}
