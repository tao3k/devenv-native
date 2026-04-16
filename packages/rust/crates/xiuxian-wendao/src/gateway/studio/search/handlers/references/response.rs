use std::sync::Arc;

use xiuxian_wendao_runtime::transport::SearchFlightRouteResponse;

use super::batch::build_reference_hits_flight_batch;
use crate::gateway::studio::{GatewayState, StudioApiError};
use crate::gateway::studio::search::handlers::queries::ReferenceSearchQuery;
use crate::gateway::studio::types::ReferenceSearchResponse;
use crate::search::SearchCorpusKind;

pub(crate) async fn load_reference_search_response(
    state: &GatewayState,
    query: ReferenceSearchQuery,
) -> Result<ReferenceSearchResponse, StudioApiError> {
    let raw_query = query.q.unwrap_or_default();
    let query_text = raw_query.trim();
    if query_text.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_QUERY",
            "Reference search requires a non-empty query",
        ));
    }
    state.studio.ensure_reference_occurrence_index_started()?;
    let status = state
        .studio
        .local_corpus_bootstrap_status(SearchCorpusKind::ReferenceOccurrence, "reference_search");
    if !status.active_epoch_ready {
        state.studio.record_local_corpus_partial_search_response(
            SearchCorpusKind::ReferenceOccurrence,
            "reference_search",
        );
        return Ok(ReferenceSearchResponse {
            query: query_text.to_string(),
            hit_count: 0,
            hits: Vec::new(),
            selected_scope: "references".to_string(),
            partial: true,
            indexing_state: Some(status.indexing_state.to_string()),
            index_error: status.index_error,
        });
    }
    let hits = state
        .studio
        .search_reference_occurrences(query_text, query.limit.unwrap_or(20).max(1))
        .await?;

    state.studio.record_local_corpus_ready_search_response(
        SearchCorpusKind::ReferenceOccurrence,
        "reference_search",
    );
    Ok(ReferenceSearchResponse {
        query: query_text.to_string(),
        hit_count: hits.len(),
        hits,
        selected_scope: "references".to_string(),
        partial: false,
        indexing_state: Some("ready".to_string()),
        index_error: None,
    })
}

pub(crate) async fn load_reference_search_flight_response(
    state: Arc<GatewayState>,
    query: ReferenceSearchQuery,
) -> Result<SearchFlightRouteResponse, StudioApiError> {
    let response = load_reference_search_response(state.as_ref(), query).await?;
    let app_metadata = serde_json::to_vec(&response).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_REFERENCE_FLIGHT_METADATA_ENCODE_FAILED",
            "Failed to encode reference-search Flight metadata",
            Some(error.to_string()),
        )
    })?;
    build_reference_hits_flight_batch(&response.hits)
        .map(|batch| SearchFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
        .map_err(|error| {
            StudioApiError::internal(
                "SEARCH_REFERENCE_FLIGHT_BATCH_BUILD_FAILED",
                "Failed to build reference-search Flight batch",
                Some(error),
            )
        })
}
