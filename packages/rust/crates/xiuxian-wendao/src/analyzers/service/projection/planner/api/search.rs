use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::service::projection::gap::build_docs_projected_gap_report;
use crate::analyzers::service::projection::planner::scoring::{
    normalize_planner_search_text, planner_gap_search_score,
};
use crate::analyzers::service::projection::registry::{
    with_bootstrapped_repository_analysis, with_repository_analysis,
};
use crate::analyzers::{
    DocsPlannerSearchHit, DocsPlannerSearchQuery, DocsPlannerSearchResult,
    DocsProjectedGapReportQuery,
};

/// Build deterministic docs-facing deep-wiki planner search hits from projected gaps.
#[must_use]
pub fn build_docs_planner_search(
    query: &DocsPlannerSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsPlannerSearchResult {
    let normalized_query = normalize_planner_search_text(query.query.as_str());
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
    .filter_map(|gap| {
        let score = planner_gap_search_score(&gap, normalized_query.as_str());
        (score > 0).then_some(DocsPlannerSearchHit {
            search_score: score,
            gap,
        })
    })
    .collect::<Vec<_>>();

    hits.sort_by(|left, right| {
        right
            .search_score
            .cmp(&left.search_score)
            .then_with(|| left.gap.kind.cmp(&right.gap.kind))
            .then_with(|| left.gap.title.cmp(&right.gap.title))
            .then_with(|| left.gap.gap_id.cmp(&right.gap.gap_id))
    });
    hits.truncate(query.limit);

    DocsPlannerSearchResult {
        repo_id: query.repo_id.clone(),
        hits,
    }
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner search hits.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_planner_search_from_config_with_registry(
    query: &DocsPlannerSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPlannerSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_planner_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner search hits.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_planner_search_from_config(
    query: &DocsPlannerSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPlannerSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_planner_search(query, analysis))
    })
}
