use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_server::transport::{
    AttachmentSearchFlightRouteProvider, AttachmentSearchFlightRouteRequest,
    SearchFlightRouteResponse,
};

use super::batch::build_attachment_hits_flight_batch;
use super::response::load_attachment_search_response_from_studio;
use crate::studio::StudioState;
use crate::studio::search::handlers::queries::AttachmentSearchQuery;

pub(crate) struct StudioAttachmentSearchFlightRouteProvider {
    studio: Arc<StudioState>,
}

impl StudioAttachmentSearchFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

impl std::fmt::Debug for StudioAttachmentSearchFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioAttachmentSearchFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AttachmentSearchFlightRouteProvider for StudioAttachmentSearchFlightRouteProvider {
    async fn attachment_search_batch_for_request(
        &self,
        request: AttachmentSearchFlightRouteRequest<'_>,
    ) -> Result<SearchFlightRouteResponse, String> {
        let mut ext = request.ext_filters.iter().cloned().collect::<Vec<_>>();
        ext.sort();
        let mut kind = request.kind_filters.iter().cloned().collect::<Vec<_>>();
        kind.sort();
        let response = load_attachment_search_response_from_studio(
            self.studio.as_ref(),
            AttachmentSearchQuery {
                q: Some(request.query_text.to_string()),
                limit: Some(request.limit),
                ext,
                kind,
                case_sensitive: request.case_sensitive,
            },
        )
        .await
        .map_err(|error| {
            error
                .error
                .details
                .clone()
                .unwrap_or_else(|| format!("{}: {}", error.code(), error.error.message))
        })?;
        let app_metadata = serde_json::to_vec(&response).map_err(|error| error.to_string())?;
        build_attachment_hits_flight_batch(&response.hits)
            .map(|batch| SearchFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
    }
}
