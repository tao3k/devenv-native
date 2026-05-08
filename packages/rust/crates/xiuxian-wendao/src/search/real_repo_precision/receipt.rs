use std::fs;
use std::path::Path;

use chrono::Utc;

use crate::search::real_repo_precision::types::{
    RealRepoGoldQueryKind, RealRepoPrecisionRepositoryReceipt, RealRepoPrecisionRunReceipt,
    RealRepoPrecisionSummary, RealRepoPrecisionSyncMode,
};

pub(crate) const RECEIPT_SCHEMA: &str = "xiuxian_wendao.real_repo_search_precision.v1";

pub(crate) fn build_run_receipt(
    sync_mode: RealRepoPrecisionSyncMode,
    query_kind_filter: Option<RealRepoGoldQueryKind>,
    repositories: Vec<RealRepoPrecisionRepositoryReceipt>,
) -> RealRepoPrecisionRunReceipt {
    let summary = summarize_repositories(&repositories);
    RealRepoPrecisionRunReceipt {
        schema: RECEIPT_SCHEMA.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        sync_mode: sync_mode.as_str().to_string(),
        query_kind_filter: query_kind_filter
            .map_or_else(|| "all".to_string(), |kind| kind.as_str().to_string()),
        summary,
        repositories,
    }
}

pub(crate) fn write_run_receipt(
    receipt: &RealRepoPrecisionRunReceipt,
    path: &Path,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create real-repo precision receipt dir `{}`: {error}",
                parent.display()
            )
        })?;
    }
    let payload = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("failed to serialize real-repo precision receipt: {error}"))?;
    fs::write(path, payload).map_err(|error| {
        format!(
            "failed to write real-repo precision receipt `{}`: {error}",
            path.display()
        )
    })
}

fn summarize_repositories(
    repositories: &[RealRepoPrecisionRepositoryReceipt],
) -> RealRepoPrecisionSummary {
    let repository_count = repositories.len();
    let materialized_repository_count = repositories.iter().filter(|repo| repo.indexed).count();
    let skipped_repository_count = repositories
        .iter()
        .filter(|repo| repo.skip_reason.is_some())
        .count();
    let query_count = repositories
        .iter()
        .map(|repo| repo.query_receipts.len())
        .sum::<usize>();
    let passed_query_count = repositories
        .iter()
        .flat_map(|repo| repo.query_receipts.iter())
        .filter(|query| query.passed)
        .count();
    let failed_query_count = query_count.saturating_sub(passed_query_count);
    let knowledge_scenario_count = repositories
        .iter()
        .map(|repo| repo.knowledge_scenarios.len())
        .sum::<usize>();
    let passed_knowledge_scenario_count = repositories
        .iter()
        .flat_map(|repo| repo.knowledge_scenarios.iter())
        .filter(|scenario| scenario.passed)
        .count();
    let failed_knowledge_scenario_count =
        knowledge_scenario_count.saturating_sub(passed_knowledge_scenario_count);
    let indexed_document_count = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.document_count)
        .sum::<usize>();
    let indexed_markdown_document_count = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.markdown_document_count)
        .sum::<usize>();
    let indexed_org_document_count = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.org_document_count)
        .sum::<usize>();
    let indexed_total_word_count = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.total_word_count)
        .sum::<usize>();

    RealRepoPrecisionSummary {
        repository_count,
        materialized_repository_count,
        skipped_repository_count,
        query_count,
        passed_query_count,
        failed_query_count,
        knowledge_scenario_count,
        passed_knowledge_scenario_count,
        failed_knowledge_scenario_count,
        indexed_document_count,
        indexed_markdown_document_count,
        indexed_org_document_count,
        indexed_total_word_count,
    }
}
