use std::sync::Arc;

use xiuxian_wendao_runtime::transport::AutocompleteFlightRouteResponse;

use super::batch::{autocomplete_response_flight_app_metadata, autocomplete_suggestion_batch};
use crate::gateway::studio::{StudioApiError, StudioState};
use crate::gateway::studio::types::{AutocompleteResponse, AutocompleteSuggestion};
use crate::search::SearchPlaneCacheTtl;

pub(crate) async fn build_autocomplete_response(
    studio: &StudioState,
    prefix: &str,
    limit: usize,
) -> Result<AutocompleteResponse, StudioApiError> {
    let prefix = prefix.trim().to_string();
    let limit = limit.max(1);
    let suggestions = if prefix.is_empty() {
        Vec::new()
    } else {
        studio.ensure_local_symbol_index_ready().await?;
        let cache_key = studio
            .search_plane
            .autocomplete_cache_key(prefix.as_str(), limit);
        if let Some(cache_key) = cache_key.as_ref()
            && let Some(cached) = studio
                .search_plane
                .cache_get_json::<Vec<AutocompleteSuggestion>>(cache_key)
                .await
        {
            return Ok(AutocompleteResponse {
                prefix,
                suggestions: cached,
            });
        }

        let suggestions = studio
            .autocomplete_local_symbols(prefix.as_str(), limit)
            .await?;
        if let Some(cache_key) = cache_key.as_ref() {
            studio
                .search_plane
                .cache_set_json(cache_key, SearchPlaneCacheTtl::Autocomplete, &suggestions)
                .await;
        }
        suggestions
    };

    Ok(AutocompleteResponse {
        prefix,
        suggestions,
    })
}

pub(super) async fn load_autocomplete_flight_response(
    studio: Arc<StudioState>,
    prefix: &str,
    limit: usize,
) -> Result<AutocompleteFlightRouteResponse, StudioApiError> {
    let response = build_autocomplete_response(studio.as_ref(), prefix, limit).await?;
    let batch = autocomplete_suggestion_batch(&response.suggestions).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_AUTOCOMPLETE_FLIGHT_BATCH_FAILED",
            "Failed to materialize autocomplete suggestions through the Flight-backed provider",
            Some(error),
        )
    })?;
    let app_metadata = autocomplete_response_flight_app_metadata(&response).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_AUTOCOMPLETE_FLIGHT_METADATA_FAILED",
            "Failed to encode autocomplete Flight app metadata",
            Some(error),
        )
    })?;
    Ok(AutocompleteFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}
