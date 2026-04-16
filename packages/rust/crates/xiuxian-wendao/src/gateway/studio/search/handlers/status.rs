use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;

use crate::gateway::studio::{
    GatewayState, StudioApiError, StudioBootstrapBackgroundIndexingTelemetry,
    StudioSearchColdStartTelemetry,
};
use crate::gateway::studio::types::SearchIndexStatusResponse;

/// Search-index status payload enriched with bootstrap-indexing telemetry.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexStatusEnvelope {
    #[serde(flatten)]
    status: SearchIndexStatusResponse,
    #[serde(flatten)]
    telemetry: StudioBootstrapBackgroundIndexingTelemetry,
    cold_start_telemetry: StudioSearchColdStartTelemetry,
}

/// Studio search-plane status endpoint.
///
/// # Errors
///
/// This handler currently does not produce handler-local errors.
pub async fn search_index_status(
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<SearchIndexStatusEnvelope>, StudioApiError> {
    let telemetry = state.studio.bootstrap_background_indexing_telemetry();
    let status = state.studio.search_index_status().await;
    Ok(Json(SearchIndexStatusEnvelope {
        status,
        telemetry,
        cold_start_telemetry: state.studio.search_cold_start_telemetry(),
    }))
}
