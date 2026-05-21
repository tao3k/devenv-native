//! `analyzers::service::projection::retrieval` owns Wendao service projection retrieval behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::{
    build_projected_retrieval, build_projected_retrieval_context, build_projected_retrieval_hit,
};
use crate::analyzers::{
    DocsRetrievalContextQuery, DocsRetrievalContextResult, DocsRetrievalQuery, DocsRetrievalResult,
    RepoProjectedRetrievalContextQuery, RepoProjectedRetrievalContextResult,
    RepoProjectedRetrievalHitQuery, RepoProjectedRetrievalHitResult, RepoProjectedRetrievalQuery,
    RepoProjectedRetrievalResult,
};
use crate::analyzers::{DocsRetrievalHitQuery, DocsRetrievalHitResult};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build deterministic docs-facing mixed retrieval results from normalized analysis records.
#[must_use]
pub fn build_docs_retrieval(
    query: &DocsRetrievalQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsRetrievalResult {
    build_repo_projected_retrieval(
        &RepoProjectedRetrievalQuery {
            repo_id: query.repo_id.clone(),
            query: query.query.clone(),
            kind: query.kind,
            limit: query.limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing mixed
/// retrieval results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_retrieval_from_config_with_registry(
    query: &DocsRetrievalQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsRetrievalResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_retrieval(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing mixed
/// retrieval results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_retrieval_from_config(
    query: &DocsRetrievalQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsRetrievalResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_retrieval(query, analysis))
    })
}

/// Build deterministic docs-facing local retrieval context around one stable hit.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page.
pub fn build_docs_retrieval_context(
    query: &DocsRetrievalContextQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
    build_repo_projected_retrieval_context(
        &RepoProjectedRetrievalContextQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
            node_id: query.node_id.clone(),
            related_limit: query.related_limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing local
/// retrieval context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn docs_retrieval_context_from_config_with_registry(
    query: &DocsRetrievalContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_retrieval_context(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing local
/// retrieval context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn docs_retrieval_context_from_config(
    query: &DocsRetrievalContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_retrieval_context(query, analysis)
    })
}

/// Build one deterministic docs-facing mixed retrieval hit from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page.
pub fn build_docs_retrieval_hit(
    query: &DocsRetrievalHitQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsRetrievalHitResult, RepoIntelligenceError> {
    build_repo_projected_retrieval_hit(
        &RepoProjectedRetrievalHitQuery {
            repo_id: query.repo.clone(),
            page_id: query.page.clone(),
            node_id: query.node.clone(),
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return one deterministic docs-facing mixed
/// retrieval hit.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn docs_retrieval_hit_from_config_with_registry(
    query: &DocsRetrievalHitQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsRetrievalHitResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo, config_path, cwd, registry, |analysis| {
        build_docs_retrieval_hit(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic docs-facing mixed
/// retrieval hit.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn docs_retrieval_hit_from_config(
    query: &DocsRetrievalHitQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsRetrievalHitResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo, config_path, cwd, |analysis| {
        build_docs_retrieval_hit(query, analysis)
    })
}

/// Build deterministic mixed retrieval results from normalized analysis records.
#[must_use]
pub fn build_repo_projected_retrieval(
    query: &RepoProjectedRetrievalQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedRetrievalResult {
    build_projected_retrieval(query, analysis)
}

/// Load configuration, analyze one repository, and return deterministic mixed retrieval results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_retrieval_from_config_with_registry(
    query: &RepoProjectedRetrievalQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedRetrievalResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_repo_projected_retrieval(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic mixed retrieval results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_retrieval_from_config(
    query: &RepoProjectedRetrievalQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedRetrievalResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_repo_projected_retrieval(query, analysis))
    })
}

/// Build one deterministic mixed retrieval hit from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page.
pub fn build_repo_projected_retrieval_hit(
    query: &RepoProjectedRetrievalHitQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedRetrievalHitResult, RepoIntelligenceError> {
    build_projected_retrieval_hit(query, analysis)
}

/// Load configuration, analyze one repository, and return one deterministic mixed retrieval hit.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn repo_projected_retrieval_hit_from_config_with_registry(
    query: &RepoProjectedRetrievalHitQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedRetrievalHitResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_retrieval_hit(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic mixed retrieval hit.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn repo_projected_retrieval_hit_from_config(
    query: &RepoProjectedRetrievalHitQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedRetrievalHitResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_retrieval_hit(query, analysis)
    })
}

/// Build deterministic local retrieval context around one stable Stage-2 hit.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page.
pub fn build_repo_projected_retrieval_context(
    query: &RepoProjectedRetrievalContextQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedRetrievalContextResult, RepoIntelligenceError> {
    build_projected_retrieval_context(query, analysis)
}

/// Load configuration, analyze one repository, and return deterministic local retrieval context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn repo_projected_retrieval_context_from_config_with_registry(
    query: &RepoProjectedRetrievalContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedRetrievalContextResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_retrieval_context(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic local retrieval context.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// hit identifiers are not present for the repository.
pub fn repo_projected_retrieval_context_from_config(
    query: &RepoProjectedRetrievalContextQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedRetrievalContextResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_retrieval_context(query, analysis)
    })
}
