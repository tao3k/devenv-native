use std::sync::Arc;

use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao_runtime::transport::{
    AutocompleteFlightRouteProvider, AutocompleteFlightRouteResponse,
};

use super::response::load_autocomplete_flight_response;
use crate::gateway::studio::{StudioApiError, StudioState};

/// Studio-backed Flight provider for the semantic `/search/autocomplete` route.
#[derive(Clone)]
pub(crate) struct StudioAutocompleteFlightRouteProvider {
    studio: Arc<StudioState>,
}

impl StudioAutocompleteFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

impl std::fmt::Debug for StudioAutocompleteFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioAutocompleteFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AutocompleteFlightRouteProvider for StudioAutocompleteFlightRouteProvider {
    async fn autocomplete_batch(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<AutocompleteFlightRouteResponse, Status> {
        load_autocomplete_flight_response(Arc::clone(&self.studio), prefix, limit)
            .await
            .map_err(studio_api_error_to_tonic_status)
    }
}

fn studio_api_error_to_tonic_status(error: StudioApiError) -> Status {
    match error.status() {
        axum::http::StatusCode::BAD_REQUEST => Status::invalid_argument(error.error.message),
        axum::http::StatusCode::NOT_FOUND => Status::not_found(error.error.message),
        axum::http::StatusCode::CONFLICT => Status::failed_precondition(error.error.message),
        _ => Status::internal(error.error.message),
    }
}
