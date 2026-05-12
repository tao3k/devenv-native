//! `analyzers::service::projection::planner::workset::orchestration` owns Wendao planner workset orchestration behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::service::projection::planner::api::{
    build_docs_planner_item, build_docs_planner_queue, build_docs_planner_rank,
};
use crate::analyzers::service::projection::planner::workset::balance::build_docs_planner_workset_balance;
use crate::analyzers::service::projection::planner::workset::groups::build_planner_workset_groups;
use crate::analyzers::service::projection::planner::workset::strategy::build_docs_planner_workset_strategy;
use crate::analyzers::service::projection::registry::{
    with_bootstrapped_repository_analysis, with_repository_analysis,
};
use crate::analyzers::{
    DocsPlannerItemQuery, DocsPlannerItemResult, DocsPlannerQueueQuery, DocsPlannerQueueResult,
    DocsPlannerRankHit, DocsPlannerRankQuery, DocsPlannerWorksetQuery, DocsPlannerWorksetResult,
};

fn planner_queue_snapshot(
    query: &DocsPlannerWorksetQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsPlannerQueueResult {
    build_docs_planner_queue(
        &DocsPlannerQueueQuery {
            repo_id: query.repo_id.clone(),
            gap_kind: query.gap_kind,
            page_kind: query.page_kind,
            per_kind_limit: query.per_kind_limit,
        },
        analysis,
    )
}

fn planner_ranked_hits(
    query: &DocsPlannerWorksetQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Vec<DocsPlannerRankHit> {
    build_docs_planner_rank(
        &DocsPlannerRankQuery {
            repo_id: query.repo_id.clone(),
            gap_kind: query.gap_kind,
            page_kind: query.page_kind,
            limit: query.limit,
        },
        analysis,
    )
    .hits
}

fn open_planner_items(
    query: &DocsPlannerWorksetQuery,
    analysis: &RepositoryAnalysisOutput,
    ranked_hits: &[DocsPlannerRankHit],
) -> Result<Vec<DocsPlannerItemResult>, RepoIntelligenceError> {
    ranked_hits
        .iter()
        .map(|ranked_hit| {
            build_docs_planner_item(
                &DocsPlannerItemQuery {
                    repo_id: query.repo_id.clone(),
                    gap_id: ranked_hit.gap.gap_id.clone(),
                    family_kind: query.family_kind,
                    related_limit: query.related_limit,
                    family_limit: query.family_limit,
                },
                analysis,
            )
        })
        .collect()
}

/// Build a deterministic docs-facing deep-wiki planner workset from ranked projected gaps.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or one selected planner item
/// cannot be reopened from the projected page kernels.
pub fn build_docs_planner_workset(
    query: &DocsPlannerWorksetQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPlannerWorksetResult, RepoIntelligenceError> {
    let queue = planner_queue_snapshot(query, analysis);
    let ranked_hits = planner_ranked_hits(query, analysis);
    let items = open_planner_items(query, analysis, &ranked_hits)?;
    let groups = build_planner_workset_groups(&ranked_hits, &items);

    let balance = build_docs_planner_workset_balance(&groups);
    let strategy = build_docs_planner_workset_strategy(&balance);

    Ok(DocsPlannerWorksetResult {
        repo_id: query.repo_id.clone(),
        queue,
        ranked_hits,
        balance,
        strategy,
        groups,
        items,
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner workset bundles.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or one selected planner item
/// cannot be reopened from the projected page kernels.
pub fn docs_planner_workset_from_config_with_registry(
    query: &DocsPlannerWorksetQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPlannerWorksetResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_planner_workset(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner workset bundles.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or one selected planner item
/// cannot be reopened from the projected page kernels.
pub fn docs_planner_workset_from_config(
    query: &DocsPlannerWorksetQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPlannerWorksetResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_planner_workset(query, analysis)
    })
}
