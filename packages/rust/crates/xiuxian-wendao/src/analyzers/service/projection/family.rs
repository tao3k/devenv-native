use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::{
    build_projected_page_family_cluster, build_projected_page_family_context,
    build_projected_page_family_search,
};
use crate::analyzers::{
    DocsFamilyClusterQuery, DocsFamilyClusterResult, DocsFamilyContextQuery,
    DocsFamilyContextResult, DocsFamilySearchQuery, DocsFamilySearchResult,
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageFamilyClusterResult,
    RepoProjectedPageFamilyContextQuery, RepoProjectedPageFamilyContextResult,
    RepoProjectedPageFamilySearchQuery, RepoProjectedPageFamilySearchResult,
};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build deterministic docs-facing page-family context around one stable projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output.
pub fn build_docs_family_context(
    query: &DocsFamilyContextQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsFamilyContextResult, RepoIntelligenceError> {
    build_repo_projected_page_family_context(
        &RepoProjectedPageFamilyContextQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
            per_kind_limit: query.per_kind_limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing
/// page-family context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn docs_family_context_from_config_with_registry(
    query: &DocsFamilyContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsFamilyContextResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_family_context(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing
/// page-family context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn docs_family_context_from_config(
    query: &DocsFamilyContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsFamilyContextResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_family_context(query, analysis)
    })
}

/// Build deterministic docs-facing page-family search results from normalized analysis records.
#[must_use]
pub fn build_docs_family_search(
    query: &DocsFamilySearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsFamilySearchResult {
    build_repo_projected_page_family_search(
        &RepoProjectedPageFamilySearchQuery {
            repo_id: query.repo_id.clone(),
            query: query.query.clone(),
            kind: query.kind,
            limit: query.limit,
            per_kind_limit: query.per_kind_limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing
/// page-family search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_family_search_from_config_with_registry(
    query: &DocsFamilySearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsFamilySearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_family_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing
/// page-family search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_family_search_from_config(
    query: &DocsFamilySearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsFamilySearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_family_search(query, analysis))
    })
}

/// Build one deterministic docs-facing page-family cluster around one stable projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or
/// [`RepoIntelligenceError::UnknownProjectedPageFamilyCluster`] when the requested family is not
/// present for the projected page.
pub fn build_docs_family_cluster(
    query: &DocsFamilyClusterQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsFamilyClusterResult, RepoIntelligenceError> {
    build_repo_projected_page_family_cluster(
        &RepoProjectedPageFamilyClusterQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
            kind: query.kind,
            limit: query.limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return one deterministic docs-facing
/// page-family cluster.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page or family cluster is not present for the repository.
pub fn docs_family_cluster_from_config_with_registry(
    query: &DocsFamilyClusterQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsFamilyClusterResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_family_cluster(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic docs-facing
/// page-family cluster.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page or family cluster is not present for the repository.
pub fn docs_family_cluster_from_config(
    query: &DocsFamilyClusterQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsFamilyClusterResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_family_cluster(query, analysis)
    })
}

/// Build deterministic page-family context around one stable projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output.
pub fn build_repo_projected_page_family_context(
    query: &RepoProjectedPageFamilyContextQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageFamilyContextResult, RepoIntelligenceError> {
    build_projected_page_family_context(query, analysis)
}

/// Load configuration, analyze one repository, and return deterministic page-family context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn repo_projected_page_family_context_from_config_with_registry(
    query: &RepoProjectedPageFamilyContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageFamilyContextResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_family_context(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic page-family context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn repo_projected_page_family_context_from_config(
    query: &RepoProjectedPageFamilyContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageFamilyContextResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_family_context(query, analysis)
    })
}

/// Build one deterministic page-family cluster around one stable projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or
/// [`RepoIntelligenceError::UnknownProjectedPageFamilyCluster`] when the requested family is not
/// present for the projected page.
pub fn build_repo_projected_page_family_cluster(
    query: &RepoProjectedPageFamilyClusterQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageFamilyClusterResult, RepoIntelligenceError> {
    build_projected_page_family_cluster(query, analysis)
}

/// Load configuration, analyze one repository, and return one deterministic page-family cluster.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page or family cluster is not present for the repository.
pub fn repo_projected_page_family_cluster_from_config_with_registry(
    query: &RepoProjectedPageFamilyClusterQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageFamilyClusterResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_family_cluster(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic page-family cluster.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page or family cluster is not present for the repository.
pub fn repo_projected_page_family_cluster_from_config(
    query: &RepoProjectedPageFamilyClusterQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageFamilyClusterResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_family_cluster(query, analysis)
    })
}

/// Build deterministic page-family search results from normalized analysis records.
#[must_use]
pub fn build_repo_projected_page_family_search(
    query: &RepoProjectedPageFamilySearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPageFamilySearchResult {
    build_projected_page_family_search(query, analysis)
}

/// Load configuration, analyze one repository, and return deterministic page-family search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_page_family_search_from_config_with_registry(
    query: &RepoProjectedPageFamilySearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageFamilySearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_repo_projected_page_family_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic page-family search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_page_family_search_from_config(
    query: &RepoProjectedPageFamilySearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageFamilySearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_repo_projected_page_family_search(query, analysis))
    })
}
