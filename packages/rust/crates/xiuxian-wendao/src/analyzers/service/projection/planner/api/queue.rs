use std::collections::BTreeMap;
use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::service::projection::gap::build_docs_projected_gap_report;
use crate::analyzers::service::projection::registry::{
    with_bootstrapped_repository_analysis, with_repository_analysis,
};
use crate::analyzers::{
    DocsPlannerQueueGroup, DocsPlannerQueueQuery, DocsPlannerQueueResult,
    DocsProjectedGapReportQuery,
};

/// Build deterministic docs-facing deep-wiki planner queue groups from projected gaps.
#[must_use]
pub fn build_docs_planner_queue(
    query: &DocsPlannerQueueQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsPlannerQueueResult {
    let gap_report = build_docs_projected_gap_report(
        &DocsProjectedGapReportQuery {
            repo_id: query.repo_id.clone(),
        },
        analysis,
    );
    let mut grouped = BTreeMap::<crate::analyzers::ProjectedGapKind, DocsPlannerQueueGroup>::new();

    for gap in gap_report.gaps.into_iter().filter(|gap| {
        query
            .gap_kind
            .is_none_or(|expected_kind| gap.kind == expected_kind)
            && query
                .page_kind
                .is_none_or(|expected_page_kind| gap.page_kind == expected_page_kind)
    }) {
        let entry = grouped
            .entry(gap.kind)
            .or_insert_with(|| DocsPlannerQueueGroup {
                kind: gap.kind,
                count: 0,
                gaps: Vec::new(),
            });
        entry.count += 1;
        if entry.gaps.len() < query.per_kind_limit {
            entry.gaps.push(gap);
        }
    }

    let mut groups = grouped.into_values().collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.kind.cmp(&right.kind))
    });

    DocsPlannerQueueResult {
        repo_id: query.repo_id.clone(),
        page_count: gap_report.summary.page_count,
        total_gap_count: groups.iter().map(|group| group.count).sum(),
        groups,
    }
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner queue groups.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_planner_queue_from_config_with_registry(
    query: &DocsPlannerQueueQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPlannerQueueResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_planner_queue(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing deep-wiki
/// planner queue groups.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn docs_planner_queue_from_config(
    query: &DocsPlannerQueueQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPlannerQueueResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_planner_queue(query, analysis))
    })
}
