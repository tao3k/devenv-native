//! `analyzers::service::projection::gap` owns Wendao service projection gap behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::build_projected_gap_report;
use crate::analyzers::{
    DocsProjectedGapReportQuery, DocsProjectedGapReportResult, RepoProjectedGapReportQuery,
    RepoProjectedGapReportResult,
};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build deterministic deep-wiki projected gap reports from normalized analysis records.
#[must_use]
pub fn build_repo_projected_gap_report(
    query: &RepoProjectedGapReportQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedGapReportResult {
    build_projected_gap_report(query, analysis)
}

/// Build deterministic docs-facing projected deep-wiki gaps from normalized analysis records.
#[must_use]
pub fn build_docs_projected_gap_report(
    query: &DocsProjectedGapReportQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsProjectedGapReportResult {
    build_repo_projected_gap_report(
        &RepoProjectedGapReportQuery {
            repo_id: query.repo_id.clone(),
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic projected deep-wiki gaps.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_gap_report_from_config_with_registry(
    query: &RepoProjectedGapReportQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedGapReportResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_repo_projected_gap_report(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic projected deep-wiki gaps.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_projected_gap_report_from_config(
    query: &RepoProjectedGapReportQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedGapReportResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_repo_projected_gap_report(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki gaps.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_projected_gap_report_from_config_with_registry(
    query: &DocsProjectedGapReportQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsProjectedGapReportResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_projected_gap_report(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki gaps.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_projected_gap_report_from_config(
    query: &DocsProjectedGapReportQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsProjectedGapReportResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_projected_gap_report(query, analysis))
    })
}
