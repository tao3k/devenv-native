use crate::studio::StudioApiError;
use crate::studio::search::handlers::knowledge::intent::types::{
    IntentIndexState, IntentMergedResults, IntentSourceHits,
};
use crate::studio::search::handlers::knowledge::intent_policy::{
    compare_intent_hits, local_symbol_hit_to_search_hit, repo_content_hit_to_intent_hit,
};
use crate::studio::types::SearchResponse;

use crate::studio::search::handlers::knowledge::merge::RepoIntentMerge;

pub(crate) fn merge_intent_hits(
    source_hits: IntentSourceHits,
    repo_merge: RepoIntentMerge,
    code_biased: bool,
) -> IntentMergedResults {
    #[cfg(test)]
    let source_transport = (
        source_hits.knowledge_query_engine,
        source_hits.local_symbol_query_engine,
    );
    let mut hits = Vec::new();
    let knowledge_hit_count = source_hits.knowledge_hits.len();
    hits.extend(source_hits.knowledge_hits);

    let local_symbol_hit_count = source_hits.local_symbol_hits.len();
    hits.extend(
        source_hits
            .local_symbol_hits
            .into_iter()
            .map(|hit| local_symbol_hit_to_search_hit(hit, code_biased)),
    );

    let repo_hit_count = repo_merge.hits.len();
    hits.extend(
        repo_merge
            .hits
            .into_iter()
            .map(|hit| repo_content_hit_to_intent_hit(hit, code_biased)),
    );
    #[cfg(test)]
    let mut transport = repo_merge.transport;
    #[cfg(not(test))]
    let transport = repo_merge.transport;
    #[cfg(test)]
    {
        transport.knowledge_query_engine = source_transport.0;
        transport.local_symbol_query_engine = source_transport.1;
    }

    IntentMergedResults {
        hits,
        knowledge_hit_count,
        local_symbol_hit_count,
        repo_hit_count,
        transport,
        partial: source_hits.knowledge_indexing
            || source_hits.local_symbol_indexing
            || !repo_merge.pending_repos.is_empty()
            || !repo_merge.skipped_repos.is_empty(),
        pending_repos: repo_merge.pending_repos,
        skipped_repos: repo_merge.skipped_repos,
    }
}

pub(crate) fn build_intent_response(
    query_text: &str,
    limit: usize,
    intent: Option<String>,
    mut merged: IntentMergedResults,
) -> SearchResponse {
    merged.hits.sort_by(compare_intent_hits);
    merged.hits.truncate(limit);

    let selected_mode = if merged.hits.is_empty() {
        "vector_only".to_string()
    } else if merged.local_symbol_hit_count > 0 || merged.repo_hit_count > 0 {
        "intent_hybrid".to_string()
    } else {
        "graph_fts".to_string()
    };
    let indexing_state = if merged.partial {
        Some(
            if merged.hits.is_empty() {
                "indexing".to_string()
            } else {
                "partial".to_string()
            }
            .into(),
        )
    } else {
        None
    };

    SearchResponse {
        query: query_text.to_string(),
        hit_count: merged.hits.len(),
        hits: merged.hits,
        graph_confidence_score: Some(if merged.knowledge_hit_count > 0 {
            1.0
        } else {
            0.0
        }),
        selected_mode: Some(selected_mode.clone().into()),
        intent,
        intent_confidence: Some(if selected_mode == "vector_only" {
            0.0
        } else {
            1.0
        }),
        search_mode: Some(selected_mode.into()),
        partial: merged.partial,
        indexing_state,
        pending_repos: merged.pending_repos,
        skipped_repos: merged.skipped_repos,
    }
}

pub(crate) fn missing_intent_config(
    index_state: &IntentIndexState,
    merged: &IntentMergedResults,
) -> bool {
    merged.hits.is_empty()
        && merged.pending_repos.is_empty()
        && merged.skipped_repos.is_empty()
        && index_state.knowledge_config_missing
        && index_state.symbol_config_missing
}

pub(crate) fn missing_intent_config_error() -> StudioApiError {
    StudioApiError::bad_request(
        "UI_CONFIG_REQUIRED",
        "Studio intent search requires configured link_graph.projects or repo_projects",
    )
}
