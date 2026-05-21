//! Bridge-audit report ingestion for materialized `SearchStrategyFlow` scenarios.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

use super::discovery::{
    discover_search_strategy_flow_candidate_inputs_with_limit, heading_sections,
    is_ignored_walk_entry, is_markdown_path, repo_relative_path,
    search_strategy_flow_candidate_input_batch,
};
use super::types::{
    MAX_CANDIDATES, SearchStrategyFlowCandidateInput, SearchStrategyFlowCandidateInputBatch,
};

const MATERIALIZED_REPO_MARKDOWN_CANDIDATE_SOURCE: &str =
    "rust-materialized-repo-markdown-headings";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowMaterializedRepoReplayFamily {
    pub(crate) repo_id: String,
    pub(crate) checkout_path: PathBuf,
    pub(crate) markdown_file_count: usize,
    pub(crate) heading_count: usize,
    pub(crate) batch: SearchStrategyFlowCandidateInputBatch,
}

pub(crate) fn materialized_search_strategy_flow_markdown_replay_families_from_bridge_report(
    report_path: &Path,
    intent: &str,
    max_repos: Option<usize>,
    max_candidates_per_repo: Option<usize>,
) -> Result<Vec<SearchStrategyFlowMaterializedRepoReplayFamily>, String> {
    let report = read_bridge_audit_report(report_path)?;
    let max_candidates = max_candidates_per_repo.unwrap_or(MAX_CANDIDATES);
    report
        .benchmark_ready_rows()
        .into_iter()
        .take(max_repos.unwrap_or(usize::MAX))
        .map(|row| materialized_replay_family(row, intent, max_candidates))
        .collect::<Result<Vec<_>, _>>()
        .map(|families| families.into_iter().flatten().collect())
}

fn read_bridge_audit_report(report_path: &Path) -> Result<BridgeAuditReport, String> {
    let text = fs::read_to_string(report_path).map_err(|error| {
        let path = report_path.display();
        format!("read SearchStrategyFlow bridge audit report {path}: {error}")
    })?;
    serde_json::from_str(&text).map_err(|error| {
        let path = report_path.display();
        format!("parse SearchStrategyFlow bridge audit report {path}: {error}")
    })
}

fn materialized_replay_family(
    row: &BridgeAuditRepoRow,
    intent: &str,
    max_candidates: usize,
) -> Result<Option<SearchStrategyFlowMaterializedRepoReplayFamily>, String> {
    let checkout_path = row.checkout_path.as_ref().ok_or_else(|| {
        format!(
            "bridge audit row `{}` is benchmark-eligible but has no checkoutPath",
            row.repo_id
        )
    })?;
    if !checkout_path.is_dir() {
        return Err(format!(
            "bridge audit row `{}` points to missing checkoutPath `{}`",
            row.repo_id,
            checkout_path.display()
        ));
    }

    let scan = scan_materialized_checkout(checkout_path)?;
    let mut candidates = discover_search_strategy_flow_candidate_inputs_with_limit(
        intent,
        checkout_path.as_path(),
        max_candidates,
    )?;
    if candidates.is_empty() {
        return Ok(None);
    }
    prefix_materialized_candidate_paths(row.repo_id.as_str(), &mut candidates);
    Ok(Some(SearchStrategyFlowMaterializedRepoReplayFamily {
        repo_id: row.repo_id.clone(),
        checkout_path: checkout_path.clone(),
        markdown_file_count: scan.markdown_file_count,
        heading_count: scan.heading_count,
        batch: search_strategy_flow_candidate_input_batch(
            MATERIALIZED_REPO_MARKDOWN_CANDIDATE_SOURCE,
            &candidates,
        ),
    }))
}

fn scan_materialized_checkout(checkout_path: &Path) -> Result<MaterializedCheckoutScan, String> {
    let markdown_files = materialized_markdown_files(checkout_path);
    let heading_count = markdown_files
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).map_err(|error| {
                let path = path.display();
                format!("read materialized SearchStrategyFlow Markdown file {path}: {error}")
            })?;
            let _relative_path = repo_relative_path(checkout_path, path)?;
            Ok::<usize, String>(heading_sections(&text).len())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    Ok(MaterializedCheckoutScan {
        markdown_file_count: markdown_files.len(),
        heading_count,
    })
}

fn materialized_markdown_files(checkout_path: &Path) -> Vec<PathBuf> {
    let mut files = WalkDir::new(checkout_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_ignored_walk_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(DirEntry::into_path)
        .filter(|path| is_markdown_path(path))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn prefix_materialized_candidate_paths(
    repo_id: &str,
    candidates: &mut [SearchStrategyFlowCandidateInput],
) {
    for candidate in candidates {
        candidate.relative_path = format!("repos/{repo_id}/{}", candidate.relative_path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MaterializedCheckoutScan {
    markdown_file_count: usize,
    heading_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeAuditReport {
    rows: Vec<BridgeAuditRepoRow>,
}

impl BridgeAuditReport {
    fn benchmark_ready_rows(&self) -> Vec<&BridgeAuditRepoRow> {
        self.rows
            .iter()
            .filter(|row| row.benchmark_eligible)
            .filter(|row| row.prewarm_action.as_deref() == Some("benchmark_ready"))
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeAuditRepoRow {
    repo_id: String,
    checkout_path: Option<PathBuf>,
    #[serde(default)]
    benchmark_eligible: bool,
    #[serde(default)]
    prewarm_action: Option<String>,
}
