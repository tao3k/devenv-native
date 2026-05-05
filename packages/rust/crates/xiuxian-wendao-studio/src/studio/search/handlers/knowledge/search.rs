//! Coordinates Studio knowledge-search responses across cache, corpus, and Flight owners.

use std::sync::Arc;

use xiuxian_wendao_server::transport::SearchFlightRouteResponse;

use super::{search_hit_batch_from_hits, search_response_flight_app_metadata};
use crate::studio::types::SearchResponse;
use crate::studio::{StudioApiError, StudioState};
use xiuxian_wendao::search::{SearchCorpusKind, SearchPlaneCacheTtl};

pub(crate) async fn build_knowledge_search_response(
    studio: &StudioState,
    query_text: &str,
    limit: usize,
    intent: Option<String>,
) -> Result<SearchResponse, StudioApiError> {
    let query_text = query_text.trim();
    if query_text.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_QUERY",
            "Knowledge search requires a non-empty query",
        ));
    }
    studio.ensure_knowledge_section_index_started()?;
    let status = studio
        .local_corpus_bootstrap_status(SearchCorpusKind::KnowledgeSection, "knowledge_search");
    if !status.active_epoch_ready {
        studio.record_local_corpus_partial_search_response(
            SearchCorpusKind::KnowledgeSection,
            "knowledge_search",
        );
        return Ok(SearchResponse {
            query: query_text.to_string(),
            hit_count: 0,
            hits: Vec::new(),
            graph_confidence_score: None,
            selected_mode: None,
            intent,
            intent_confidence: None,
            search_mode: None,
            partial: true,
            indexing_state: Some(status.indexing_state.to_string()),
            pending_repos: Vec::new(),
            skipped_repos: Vec::new(),
        });
    }
    let cache_key = studio.search_plane.search_query_cache_key(
        "knowledge",
        &[SearchCorpusKind::KnowledgeSection],
        query_text,
        limit,
        intent.as_deref(),
        None,
    );
    if let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = studio
            .search_plane
            .cache_get_json::<SearchResponse>(cache_key)
            .await
    {
        return Ok(cached);
    }
    let hits = studio.search_knowledge_sections(query_text, limit).await?;

    let selected_mode = if hits.is_empty() {
        "vector_only".to_string()
    } else {
        "graph_fts".to_string()
    };
    let graph_confidence_score = if hits.is_empty() { 0.0 } else { 1.0 };
    let response = SearchResponse {
        query: query_text.to_string(),
        hit_count: hits.len(),
        hits,
        graph_confidence_score: Some(graph_confidence_score),
        selected_mode: Some(selected_mode.clone()),
        intent,
        intent_confidence: Some(graph_confidence_score),
        search_mode: Some(selected_mode),
        partial: false,
        indexing_state: Some("ready".to_string()),
        pending_repos: Vec::new(),
        skipped_repos: Vec::new(),
    };
    studio.record_local_corpus_ready_search_response(
        SearchCorpusKind::KnowledgeSection,
        "knowledge_search",
    );
    if let Some(cache_key) = cache_key.as_ref() {
        studio
            .search_plane
            .cache_set_json(cache_key, SearchPlaneCacheTtl::HotQuery, &response)
            .await;
    }
    Ok(response)
}

pub(crate) async fn load_knowledge_search_flight_response(
    studio: Arc<StudioState>,
    query_text: &str,
    limit: usize,
) -> Result<SearchFlightRouteResponse, StudioApiError> {
    let response = build_knowledge_search_response(
        studio.as_ref(),
        query_text,
        limit,
        Some("semantic_lookup".to_string()),
    )
    .await?;
    let batch = search_hit_batch_from_hits(&response.hits).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_KNOWLEDGE_FLIGHT_BATCH_FAILED",
            "Failed to materialize knowledge hits through the Flight-backed provider",
            Some(error),
        )
    })?;
    let app_metadata = search_response_flight_app_metadata(&response).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_KNOWLEDGE_FLIGHT_METADATA_FAILED",
            "Failed to encode knowledge Flight app metadata",
            Some(error),
        )
    })?;
    Ok(SearchFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}
