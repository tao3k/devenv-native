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
    let repositories_total = repositories.len();
    let repositories_materialized = repositories.iter().filter(|repo| repo.indexed).count();
    let repositories_skipped = repositories
        .iter()
        .filter(|repo| repo.skip_reason.is_some())
        .count();
    let queries_total = repositories
        .iter()
        .map(|repo| repo.query_receipts.len())
        .sum::<usize>();
    let queries_passed = repositories
        .iter()
        .flat_map(|repo| repo.query_receipts.iter())
        .filter(|query| query.passed)
        .count();
    let queries_failed = queries_total.saturating_sub(queries_passed);
    let knowledge_scenarios_total = repositories
        .iter()
        .map(|repo| repo.knowledge_scenarios.len())
        .sum::<usize>();
    let knowledge_scenarios_passed = repositories
        .iter()
        .flat_map(|repo| repo.knowledge_scenarios.iter())
        .filter(|scenario| scenario.passed)
        .count();
    let knowledge_scenarios_failed =
        knowledge_scenarios_total.saturating_sub(knowledge_scenarios_passed);
    let indexed_documents = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.document_count)
        .sum::<usize>();
    let indexed_markdown_documents = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.markdown_document_count)
        .sum::<usize>();
    let indexed_org_documents = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.org_document_count)
        .sum::<usize>();
    let indexed_total_words = repositories
        .iter()
        .filter_map(|repo| repo.link_graph_corpus.as_ref())
        .map(|corpus| corpus.total_word_count)
        .sum::<usize>();

    RealRepoPrecisionSummary {
        repositories_total,
        repositories_materialized,
        repositories_skipped,
        queries_total,
        queries_passed,
        queries_failed,
        knowledge_scenarios_total,
        knowledge_scenarios_passed,
        knowledge_scenarios_failed,
        indexed_documents,
        indexed_markdown_documents,
        indexed_org_documents,
        indexed_total_words,
    }
}
