use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::{
    build_projected_page_navigation, build_projected_page_navigation_search,
};
use crate::analyzers::{
    DocsNavigationQuery, DocsNavigationResult, RepoProjectedPageNavigationQuery,
    RepoProjectedPageNavigationResult, RepoProjectedPageNavigationSearchQuery,
    RepoProjectedPageNavigationSearchResult,
};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build one docs-facing deterministic page-centric navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page, or
/// [`RepoIntelligenceError::UnknownProjectedPageFamilyCluster`] when the requested family is not
/// present for the projected page.
pub fn build_docs_navigation(
    query: &DocsNavigationQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsNavigationResult, RepoIntelligenceError> {
    build_repo_projected_page_navigation(
        &RepoProjectedPageNavigationQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
            node_id: query.node_id.clone(),
            family_kind: query.family_kind,
            related_limit: query.related_limit,
            family_limit: query.family_limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic
/// page-centric navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page, node, or family cluster is not present for the repository.
pub fn docs_navigation_from_config_with_registry(
    query: &DocsNavigationQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsNavigationResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_navigation(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic
/// page-centric navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page, node, or family cluster is not present for the repository.
pub fn docs_navigation_from_config(
    query: &DocsNavigationQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsNavigationResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_navigation(query, analysis)
    })
}

/// Build deterministic docs-facing projected page-navigation search results from normalized
/// analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when a matched projected page cannot be expanded into a
/// deterministic navigation bundle.
pub fn build_docs_navigation_search(
    query: &crate::analyzers::DocsNavigationSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<crate::analyzers::DocsNavigationSearchResult, RepoIntelligenceError> {
    build_repo_projected_page_navigation_search(
        &RepoProjectedPageNavigationSearchQuery {
            repo_id: query.repo_id.clone(),
            query: query.query.clone(),
            kind: query.kind,
            family_kind: query.family_kind,
            limit: query.limit,
            related_limit: query.related_limit,
            family_limit: query.family_limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-navigation search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or a matched projected page
/// cannot be expanded into a deterministic navigation bundle.
pub fn docs_navigation_search_from_config_with_registry(
    query: &crate::analyzers::DocsNavigationSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<crate::analyzers::DocsNavigationSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_navigation_search(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-navigation search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or a matched projected page
/// cannot be expanded into a deterministic navigation bundle.
pub fn docs_navigation_search_from_config(
    query: &crate::analyzers::DocsNavigationSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<crate::analyzers::DocsNavigationSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_navigation_search(query, analysis)
    })
}

/// Build one deterministic page-centric Stage-2 navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page, or
/// [`RepoIntelligenceError::UnknownProjectedPageFamilyCluster`] when the requested family is not
/// present for the projected page.
pub fn build_repo_projected_page_navigation(
    query: &RepoProjectedPageNavigationQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageNavigationResult, RepoIntelligenceError> {
    build_projected_page_navigation(query, analysis)
}

/// Load configuration, analyze one repository, and return one deterministic page-centric
/// Stage-2 navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page, node, or family cluster is not present for the repository.
pub fn repo_projected_page_navigation_from_config_with_registry(
    query: &RepoProjectedPageNavigationQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageNavigationResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_navigation(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic page-centric
/// Stage-2 navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page, node, or family cluster is not present for the repository.
pub fn repo_projected_page_navigation_from_config(
    query: &RepoProjectedPageNavigationQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageNavigationResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_navigation(query, analysis)
    })
}

/// Build deterministic projected page-navigation search results from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when a matched projected page cannot be expanded into a
/// deterministic navigation bundle.
pub fn build_repo_projected_page_navigation_search(
    query: &RepoProjectedPageNavigationSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageNavigationSearchResult, RepoIntelligenceError> {
    build_projected_page_navigation_search(query, analysis)
}

/// Load configuration, analyze one repository, and return deterministic projected page-navigation
/// search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or a matched projected page
/// cannot be expanded into a deterministic navigation bundle.
pub fn repo_projected_page_navigation_search_from_config_with_registry(
    query: &RepoProjectedPageNavigationSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageNavigationSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_navigation_search(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic projected page-navigation
/// search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or a matched projected page
/// cannot be expanded into a deterministic navigation bundle.
pub fn repo_projected_page_navigation_search_from_config(
    query: &RepoProjectedPageNavigationSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageNavigationSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_navigation_search(query, analysis)
    })
}
