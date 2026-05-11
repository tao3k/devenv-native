use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::service::projection::gap::build_docs_projected_gap_report;
use crate::analyzers::service::projection::navigation::build_docs_navigation;
use crate::analyzers::service::projection::registry::{
    with_bootstrapped_repository_analysis, with_repository_analysis,
};
use crate::analyzers::service::projection::retrieval::build_docs_retrieval_hit;
use crate::analyzers::{
    DocsNavigationQuery, DocsPlannerItemQuery, DocsPlannerItemResult, DocsProjectedGapReportQuery,
};

/// Build one deterministic docs-facing deep-wiki planner item from a stable projected gap.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedGap`] when the requested projected gap is not
/// present in the analysis output, or propagates the deterministic navigation and retrieval-hit
/// lookup errors for the owning projected page.
pub fn build_docs_planner_item(
    query: &DocsPlannerItemQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPlannerItemResult, RepoIntelligenceError> {
    let gap_report = build_docs_projected_gap_report(
        &DocsProjectedGapReportQuery {
            repo_id: query.repo_id.clone(),
        },
        analysis,
    );
    let gap = gap_report
        .gaps
        .into_iter()
        .find(|gap| gap.gap_id == query.gap_id)
        .ok_or_else(|| RepoIntelligenceError::UnknownProjectedGap {
            repo_id: query.repo_id.clone().into(),
            gap_id: query.gap_id.clone().into(),
        })?;
    let hit = build_docs_retrieval_hit(
        &crate::analyzers::DocsRetrievalHitQuery {
            repo: query.repo_id.clone(),
            page: gap.page_id.clone(),
            node: None,
        },
        analysis,
    )?
    .hit;
    let navigation = build_docs_navigation(
        &DocsNavigationQuery {
            repo_id: query.repo_id.clone(),
            page_id: gap.page_id.clone(),
            node_id: None,
            family_kind: query.family_kind,
            related_limit: query.related_limit,
            family_limit: query.family_limit,
        },
        analysis,
    )?;

    Ok(DocsPlannerItemResult {
        repo_id: query.repo_id.clone(),
        gap,
        hit,
        navigation,
    })
}

/// Load configuration, analyze one repository, and return one deterministic docs-facing deep-wiki
/// planner item.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected gap
/// or owning projected page identifiers are not present for the repository.
pub fn docs_planner_item_from_config_with_registry(
    query: &DocsPlannerItemQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPlannerItemResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_planner_item(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic docs-facing deep-wiki
/// planner item.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected gap
/// or owning projected page identifiers are not present for the repository.
pub fn docs_planner_item_from_config(
    query: &DocsPlannerItemQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPlannerItemResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_planner_item(query, analysis)
    })
}
