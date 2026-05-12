use std::cmp::Ordering;
use std::collections::HashSet;
use std::time::Instant;

use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::REPO_SEARCH_ROUTE;

use crate::integration_support::search_strategy_flow_candidates::{
    CODE_INTELLIGENCE_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};

use super::client::SearchStrategyFlowFlightClient;
use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::constants::{
    MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS, MAX_FLIGHT_DISCOVERY_CANDIDATES,
    MIN_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP, REPO_SEARCH_LIMIT,
};
use super::metadata::populate_repo_search_headers;
use super::query::candidate_discovery_queries;
use super::rows::{
    repo_relative_candidate_inputs, repo_search_batches_to_candidate_inputs, row_count,
};

pub(crate) async fn search_strategy_flow_candidate_input_batch_from_repo_search(
    intent: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let discovery_started_at = Instant::now();
    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    let attempts = candidate_discovery_queries(intent);
    let mut attempted = Vec::new();
    let mut attempt_receipts = Vec::new();
    let mut seen = HashSet::<(String, String)>::new();
    let mut merged_candidates = Vec::new();
    for (attempt_index, attempt) in attempts
        .iter()
        .take(MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS)
        .enumerate()
    {
        attempted.push(format!(
            "query=`{}` prefix=`{}`",
            attempt.query, attempt.path_prefix
        ));
        let attempt_started_at = Instant::now();
        let batches = client
            .collect_route_batches_allow_empty(
                REPO_SEARCH_ROUTE,
                "SearchStrategyFlow repo-search candidate discovery",
                |metadata| {
                    populate_repo_search_headers(
                        metadata,
                        &config.repo_id,
                        attempt.query.as_str(),
                        REPO_SEARCH_LIMIT,
                        attempt.path_prefix.as_str(),
                    )
                },
            )
            .await?;
        attempt_receipts.push(candidate_discovery_attempt_receipt(
            attempt.query.as_str(),
            attempt.path_prefix.as_str(),
            row_count(&batches),
            elapsed_ms(attempt_started_at),
        ));
        for candidate in repo_relative_candidate_inputs(
            config.repo_id.as_str(),
            repo_search_batches_to_candidate_inputs(&batches),
        ) {
            let key = (
                candidate.relative_path.clone(),
                candidate.heading_anchor.clone(),
            );
            if seen.insert(key) {
                merged_candidates.push(candidate);
            }
        }
        if should_stop_candidate_discovery(attempt_index + 1, &merged_candidates) {
            break;
        }
    }
    if !merged_candidates.is_empty() {
        calibrate_candidate_discovery_scores(&mut merged_candidates);
        rank_candidate_discovery_results(&mut merged_candidates);
        retain_unique_candidate_sources(&mut merged_candidates);
        merged_candidates.truncate(MAX_FLIGHT_DISCOVERY_CANDIDATES);
        return Ok(
            search_strategy_flow_candidate_input_batch_with_discovery_receipt(
                CODE_INTELLIGENCE_CANDIDATE_SOURCE,
                &merged_candidates,
                candidate_discovery_receipt(
                    config.repo_id.as_str(),
                    merged_candidates.len(),
                    elapsed_ms(discovery_started_at),
                    attempt_receipts,
                ),
            ),
        );
    }

    Err(format!(
        "SearchStrategyFlow repo-search candidate discovery returned zero page-index-ready candidate rows after {} attempts: {}",
        attempted.len(),
        attempted.join("; ")
    ))
}

fn rank_candidate_discovery_results(candidates: &mut [SearchStrategyFlowCandidateInput]) {
    candidates.sort_by(compare_candidate_discovery_results);
}

fn retain_unique_candidate_sources(candidates: &mut Vec<SearchStrategyFlowCandidateInput>) {
    let mut seen_paths = HashSet::<String>::new();
    candidates.retain(|candidate| seen_paths.insert(candidate.relative_path.clone()));
}

fn should_stop_candidate_discovery(
    attempted_count: usize,
    candidates: &[SearchStrategyFlowCandidateInput],
) -> bool {
    attempted_count >= MIN_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP
        && unique_candidate_source_count(candidates) >= MAX_FLIGHT_DISCOVERY_CANDIDATES
}

fn unique_candidate_source_count(candidates: &[SearchStrategyFlowCandidateInput]) -> usize {
    candidates
        .iter()
        .map(|candidate| candidate.relative_path.as_str())
        .collect::<HashSet<_>>()
        .len()
}

fn calibrate_candidate_discovery_scores(candidates: &mut [SearchStrategyFlowCandidateInput]) {
    for candidate in candidates {
        match candidate_discovery_priority(candidate) {
            0 => apply_candidate_score_floor(candidate, 0.97, 0.96, 0.97, 0.94, 0.05),
            1 => apply_candidate_score_floor(candidate, 0.95, 0.94, 0.95, 0.92, 0.07),
            2 => apply_candidate_score_floor(candidate, 0.93, 0.92, 0.93, 0.90, 0.08),
            _ => {}
        }
    }
}

fn apply_candidate_score_floor(
    candidate: &mut SearchStrategyFlowCandidateInput,
    evidence_coverage: f64,
    graph_score: f64,
    authority_score: f64,
    structural_score: f64,
    uncertainty_ceiling: f64,
) {
    candidate.evidence_coverage = candidate.evidence_coverage.max(evidence_coverage);
    candidate.graph_score = candidate.graph_score.max(graph_score);
    candidate.authority_score = candidate.authority_score.max(authority_score);
    candidate.structural_score = candidate.structural_score.max(structural_score);
    candidate.uncertainty = candidate.uncertainty.min(uncertainty_ceiling);
}

fn compare_candidate_discovery_results(
    left: &SearchStrategyFlowCandidateInput,
    right: &SearchStrategyFlowCandidateInput,
) -> Ordering {
    candidate_discovery_priority(left)
        .cmp(&candidate_discovery_priority(right))
        .then_with(|| compare_score(right.evidence_coverage, left.evidence_coverage))
        .then_with(|| compare_score(right.graph_score, left.graph_score))
        .then_with(|| left.relative_path.cmp(&right.relative_path))
        .then_with(|| left.heading_anchor.cmp(&right.heading_anchor))
}

fn compare_score(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

fn candidate_discovery_priority(candidate: &SearchStrategyFlowCandidateInput) -> u8 {
    let path = candidate.relative_path.to_ascii_lowercase();
    let title = candidate.title.to_ascii_lowercase();
    let combined = format!("{path} {title}");
    if is_test_path(path.as_str()) {
        return 6;
    }
    if is_search_strategy_flow_owner_markdown_path(path.as_str()) {
        return 0;
    }
    if is_validation_authority_markdown_path(path.as_str(), combined.as_str()) {
        return 1;
    }
    if is_policy_authority_markdown_path(path.as_str(), combined.as_str()) {
        return 2;
    }
    if is_markdown_path(path.as_str()) {
        return 3;
    }
    if path.ends_with(".toml") {
        return 4;
    }
    if is_package_source_path(path.as_str()) {
        return 5;
    }
    6
}

fn is_search_strategy_flow_owner_markdown_path(path: &str) -> bool {
    is_markdown_path(path)
        && (path == "packages/rust/crates/xiuxian-wendao-julia/readme.md"
            || path.starts_with("packages/rust/crates/xiuxian-wendao-julia/docs/"))
}

fn is_policy_authority_markdown_path(path: &str, combined: &str) -> bool {
    is_markdown_path(path)
        && ((path.starts_with("docs/rfcs/") && has_authority_terms(combined))
            || (path.starts_with("docs/01_core/wendao/")
                && has_search_strategy_terms(combined)
                && has_relation_terms(combined)))
}

fn is_validation_authority_markdown_path(path: &str, combined: &str) -> bool {
    is_markdown_path(path) && path.starts_with("docs/testing/") && combined.contains("validation")
}

fn has_authority_terms(combined: &str) -> bool {
    combined.contains("ownership")
        || combined.contains("authority")
        || combined.contains("validation")
        || combined.contains("boundary")
}

fn has_search_strategy_terms(combined: &str) -> bool {
    combined.contains("searchstrategyflow")
        || combined.contains("search strategy")
        || combined.contains("pageindex")
        || combined.contains("page index")
}

fn has_relation_terms(combined: &str) -> bool {
    combined.contains("linkgraph")
        || combined.contains("link graph")
        || combined.contains("relation")
}

fn is_markdown_path(path: &str) -> bool {
    path.ends_with(".md")
}

fn is_package_source_path(path: &str) -> bool {
    path.starts_with("packages/") && !is_test_path(path)
}

fn is_test_path(path: &str) -> bool {
    path.contains("/tests/")
        || path.starts_with("tests/")
        || path.contains("/test/")
        || path.ends_with("_test.rs")
        || path.ends_with(".test.ts")
        || path.ends_with(".spec.ts")
}

fn candidate_discovery_attempt_receipt(
    query: &str,
    path_prefix: &str,
    row_count: usize,
    elapsed_ms: u128,
) -> Value {
    json!({
        "route": REPO_SEARCH_ROUTE,
        "query": query,
        "pathPrefix": path_prefix,
        "requestLimit": REPO_SEARCH_LIMIT,
        "rowCount": row_count,
        "elapsedMs": elapsed_ms,
    })
}

fn candidate_discovery_receipt(
    repo_id: &str,
    merged_candidate_count: usize,
    elapsed_ms: u128,
    attempts: Vec<Value>,
) -> Value {
    json!({
        "receiptSource": CODE_INTELLIGENCE_CANDIDATE_SOURCE,
        "candidateInputSource": CODE_INTELLIGENCE_CANDIDATE_SOURCE,
        "candidateInputCount": merged_candidate_count,
        "repoId": repo_id,
        "transport": "arrow-flight",
        "route": REPO_SEARCH_ROUTE,
        "requestLimit": REPO_SEARCH_LIMIT,
        "attemptCount": attempts.len(),
        "maxAttemptCount": MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS,
        "maxMergedCandidateCount": MAX_FLIGHT_DISCOVERY_CANDIDATES,
        "mergedCandidateCount": merged_candidate_count,
        "elapsedMs": elapsed_ms,
        "attempts": attempts,
    })
}

fn elapsed_ms(started_at: Instant) -> u128 {
    started_at.elapsed().as_millis()
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/candidate_source.rs"]
mod tests;
