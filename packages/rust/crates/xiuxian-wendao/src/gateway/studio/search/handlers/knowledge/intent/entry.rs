use crate::gateway::studio::{StudioApiError, StudioState};
use crate::gateway::studio::search::handlers::code_search::build_code_search_response;
use crate::gateway::studio::search::handlers::knowledge::helpers::{
    intent_candidate_limit, is_code_biased_intent,
};
use crate::gateway::studio::search::handlers::knowledge::intent::cache::build_intent_cache_key;
use crate::gateway::studio::search::handlers::knowledge::intent::indices::ensure_intent_indices;
use crate::gateway::studio::search::handlers::knowledge::intent::response::{
    build_intent_response, merge_intent_hits, missing_intent_config, missing_intent_config_error,
};
use crate::gateway::studio::search::handlers::knowledge::intent::sources::search_intent_sources;
use crate::gateway::studio::search::handlers::knowledge::intent::types::IntentSearchTransportMetadata;
#[cfg(test)]
use crate::gateway::studio::search::handlers::queries::SearchQuery;
use crate::gateway::studio::types::SearchResponse;
use crate::search::SearchPlaneCacheTtl;

#[cfg(test)]
pub(crate) async fn load_intent_search_response_with_metadata(
    studio: &StudioState,
    query: SearchQuery,
) -> Result<(SearchResponse, IntentSearchTransportMetadata), StudioApiError> {
    let raw_query = query.q.unwrap_or_default();
    let query_text = raw_query.trim();
    let intent = query.intent.clone().unwrap_or_default();
    let limit = query.limit.unwrap_or(10).max(1);

    if query_text.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_QUERY",
            "Intent search requires a non-empty query",
        ));
    }

    build_intent_search_response_with_metadata(
        studio,
        raw_query.as_str(),
        query_text,
        query.repo.as_deref(),
        limit,
        (!intent.is_empty()).then_some(intent),
    )
    .await
}

#[cfg(test)]
pub async fn build_intent_search_response(
    studio: &StudioState,
    raw_query: &str,
    query_text: &str,
    repo_hint: Option<&str>,
    limit: usize,
    intent: Option<String>,
) -> Result<SearchResponse, StudioApiError> {
    build_intent_search_response_with_metadata(
        studio, raw_query, query_text, repo_hint, limit, intent,
    )
    .await
    .map(|(response, _)| response)
}

pub(crate) async fn build_intent_search_response_with_metadata(
    studio: &StudioState,
    raw_query: &str,
    query_text: &str,
    repo_hint: Option<&str>,
    limit: usize,
    intent: Option<String>,
) -> Result<(SearchResponse, IntentSearchTransportMetadata), StudioApiError> {
    if intent.as_deref() == Some("code_search") {
        return build_code_search_response(studio, raw_query.to_string(), repo_hint, limit)
            .await
            .map(|response| (response, IntentSearchTransportMetadata::default()));
    }

    let index_state = ensure_intent_indices(studio);
    let candidate_limit = intent_candidate_limit(limit);
    let intent_ref = intent.as_deref();
    let code_biased = is_code_biased_intent(intent_ref, query_text, repo_hint);
    let cache_key = build_intent_cache_key(
        studio,
        raw_query,
        query_text,
        repo_hint,
        limit,
        intent_ref,
        code_biased,
    )
    .await?;
    if let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = studio
            .search_plane
            .cache_get_json::<SearchResponse>(cache_key)
            .await
    {
        return Ok((cached, IntentSearchTransportMetadata::default()));
    }
    let source_hits =
        search_intent_sources(studio, query_text, candidate_limit, &index_state).await?;

    let repo_merge = if code_biased {
        crate::gateway::studio::search::handlers::knowledge::merge::build_repo_intent_merge(
            studio,
            raw_query,
            repo_hint,
            candidate_limit,
        )
        .await?
    } else {
        crate::gateway::studio::search::handlers::knowledge::merge::RepoIntentMerge::default()
    };

    let merged = merge_intent_hits(source_hits, repo_merge, code_biased);
    if missing_intent_config(&index_state, &merged) {
        return Err(missing_intent_config_error());
    }

    let transport = merged.transport.clone();
    let response = build_intent_response(query_text, limit, intent, merged);
    if !response.partial
        && let Some(cache_key) = cache_key.as_ref()
    {
        studio
            .search_plane
            .cache_set_json(cache_key, SearchPlaneCacheTtl::HotQuery, &response)
            .await;
    }
    Ok((response, transport))
}
