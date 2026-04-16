use std::sync::Arc;

use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao_runtime::transport::{
    DefinitionFlightRouteProvider, DefinitionFlightRouteResponse,
};

use super::response::load_definition_flight_response;
use crate::gateway::studio::{StudioApiError, StudioState};

/// Studio-backed Flight provider for the semantic `/search/definition` route.
#[derive(Clone)]
pub(crate) struct StudioDefinitionFlightRouteProvider {
    studio: Arc<StudioState>,
}

impl StudioDefinitionFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

impl std::fmt::Debug for StudioDefinitionFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioDefinitionFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl DefinitionFlightRouteProvider for StudioDefinitionFlightRouteProvider {
    async fn definition_batch(
        &self,
        query_text: &str,
        source_path: Option<&str>,
        source_line: Option<usize>,
    ) -> Result<DefinitionFlightRouteResponse, Status> {
        load_definition_flight_response(
            Arc::clone(&self.studio),
            query_text,
            source_path,
            source_line,
        )
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
