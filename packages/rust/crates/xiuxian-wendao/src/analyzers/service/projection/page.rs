//! `analyzers::service::projection::page` owns Wendao service projection page behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::{build_projected_page, render_projected_markdown_documents};
use crate::analyzers::{
    DocsMarkdownDocumentsQuery, DocsMarkdownDocumentsResult, DocsPageQuery, DocsPageResult,
    RepoProjectedPageQuery, RepoProjectedPageResult,
};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build deterministic docs-facing projected markdown documents from normalized analysis records.
#[must_use]
pub fn build_docs_markdown_documents(
    query: &DocsMarkdownDocumentsQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsMarkdownDocumentsResult {
    DocsMarkdownDocumentsResult {
        repo_id: query.repo_id.clone(),
        documents: render_projected_markdown_documents(analysis),
    }
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// markdown documents.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_markdown_documents_from_config_with_registry(
    query: &DocsMarkdownDocumentsQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsMarkdownDocumentsResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_markdown_documents(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// markdown documents.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_markdown_documents_from_config(
    query: &DocsMarkdownDocumentsQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsMarkdownDocumentsResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_markdown_documents(query, analysis))
    })
}

/// Build one docs-facing deterministic projected page from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output.
pub fn build_docs_page(
    query: &DocsPageQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPageResult, RepoIntelligenceError> {
    build_repo_projected_page(
        &RepoProjectedPageQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn docs_page_from_config_with_registry(
    query: &DocsPageQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPageResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_page(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn docs_page_from_config(
    query: &DocsPageQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPageResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_page(query, analysis)
    })
}

/// Build one deterministic projected page from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output.
pub fn build_repo_projected_page(
    query: &RepoProjectedPageQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageResult, RepoIntelligenceError> {
    build_projected_page(query, analysis)
}

/// Load configuration, analyze one repository, and return one deterministic projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn repo_projected_page_from_config_with_registry(
    query: &RepoProjectedPageQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page identifier is not present for the repository.
pub fn repo_projected_page_from_config(
    query: &RepoProjectedPageQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page(query, analysis)
    })
}
