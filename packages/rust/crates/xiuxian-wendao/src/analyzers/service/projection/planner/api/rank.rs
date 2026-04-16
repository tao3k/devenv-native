use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::service::projection::gap::build_docs_projected_gap_report;
use crate::analyzers::service::projection::planner::scoring::planner_gap_priority_breakdown;
use crate::analyzers::service::projection::registry::{
    with_bootstrapped_repository_analysis, with_repository_analysis,
};
use crate::analyzers::{
    DocsPlannerRankHit, DocsPlannerRankQuery, DocsPlannerRankResult, DocsProjectedGapReportQuery,
};

/// Build deterministic docs-facing deep-wiki planner ranking hits from projected gaps.
#[must_use]
pub fn build_docs_planner_rank(
    query: &DocsPlannerRankQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsPlannerRankResult {
    let mut hits = build_docs_projected_gap_report(
        &DocsProjectedGapReportQuery {
            repo_id: query.repo_id.clone(),
        },
        analysis,
    )
    .gaps
    .into_iter()
    .filter(|gap| {
        query
            .gap_kind
            .is_none_or(|expected_kind| gap.kind == expected_kind)
            && query
                .page_kind
                .is_none_or(|expected_page_kind| gap.page_kind == expected_page_kind)
    })
    .map(|gap| {
        let (priority_score, reasons) = planner_gap_priority_breakdown(&gap);
        DocsPlannerRankHit {
            priority_score,
            reasons,
            gap,
        }
    })
    .collect::<Vec<_>>();

    hits.sort_by(|left, right| {
        right
            .priority_score
            .cmp(&left.priority_score)
            .then_with(|| left.gap.kind.cmp(&right.gap.kind))
            .then_with(|| left.gap.title.cmp(&right.gap.title))
            .then_with(|| left.gap.gap_id.cmp(&right.gap.gap_id))
    });
    hits.truncate(query.limit);

    DocsPlannerRankResult {
        repo_id: query.repo_id.clone(),
        hits,
    }
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner ranking hits.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_planner_rank_from_config_with_registry(
    query: &DocsPlannerRankQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPlannerRankResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_planner_rank(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner ranking hits.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_planner_rank_from_config(
    query: &DocsPlannerRankQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPlannerRankResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_planner_rank(query, analysis))
    })
}
