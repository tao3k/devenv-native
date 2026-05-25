use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInput;

use super::path::path_has_extension;
use crate::integration_support::search_strategy_flow_flight::query::RepoSearchAttempt;

pub(super) fn candidate_from_exact_markdown_attempt(
    attempt: &RepoSearchAttempt,
) -> Option<SearchStrategyFlowCandidateInput> {
    if !attempt_is_intent_exact_markdown_seed(attempt) {
        return None;
    }
    let title = exact_markdown_seed_title(attempt);
    Some(SearchStrategyFlowCandidateInput {
        relative_path: attempt.path_prefix.clone(),
        heading_anchor: "document".to_owned(),
        title,
        line_start: 1,
        line_end: 1,
        context_cost: 8,
        evidence_coverage: 0.98,
        graph_score: 0.96,
        authority_score: 0.95,
        structural_score: 0.94,
        uncertainty: 0.04,
        blocked: false,
        edge_kinds: vec!["intent-exact-markdown-seed".to_owned()],
    })
}

pub(super) fn apply_exact_markdown_attempt_score_floor(
    candidates: &mut [SearchStrategyFlowCandidateInput],
    attempt: &RepoSearchAttempt,
) {
    if !attempt_is_intent_exact_markdown_seed(attempt) {
        return;
    }
    for candidate in candidates
        .iter_mut()
        .filter(|candidate| candidate.relative_path == attempt.path_prefix)
    {
        super::ranking::apply_candidate_score_floor(candidate, 0.98, 0.96, 0.95, 0.94, 0.04);
        if !candidate
            .edge_kinds
            .iter()
            .any(|kind| kind == "intent-exact-markdown-seed")
        {
            candidate
                .edge_kinds
                .push("intent-exact-markdown-seed".to_owned());
        }
    }
}

fn attempt_is_intent_exact_markdown_seed(attempt: &RepoSearchAttempt) -> bool {
    if attempt.path_prefix.trim().is_empty()
        || !path_has_extension(attempt.path_prefix.as_str(), "md")
    {
        return false;
    }
    !matches!(
        attempt.query.trim().to_ascii_lowercase().as_str(),
        "searchstrategyflow" | "pageindex" | "linkgraph"
    )
}

fn exact_markdown_seed_title(attempt: &RepoSearchAttempt) -> String {
    let file_stem = std::path::Path::new(attempt.path_prefix.as_str())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(attempt.path_prefix.as_str())
        .replace(['_', '-'], " ");
    if attempt.query.trim().is_empty() {
        file_stem
    } else {
        format!("{file_stem}: {}", attempt.query.trim())
    }
}
