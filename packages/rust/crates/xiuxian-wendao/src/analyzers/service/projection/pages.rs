//! `analyzers::service::projection::pages` owns Wendao service projection pages behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::build_projected_pages;
use crate::analyzers::{RepoProjectedPagesQuery, RepoProjectedPagesResult};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build deterministic projected pages from normalized analysis records.
#[must_use]
pub fn build_repo_projected_pages(
    query: &RepoProjectedPagesQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPagesResult {
    RepoProjectedPagesResult {
        repo_id: query.repo_id.clone(),
        pages: build_projected_pages(analysis),
    }
}

/// Load configuration, analyze one repository, and return deterministic projected pages.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_pages_from_config_with_registry(
    query: &RepoProjectedPagesQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPagesResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_repo_projected_pages(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic projected pages.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_pages_from_config(
    query: &RepoProjectedPagesQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPagesResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_repo_projected_pages(query, analysis))
    })
}
