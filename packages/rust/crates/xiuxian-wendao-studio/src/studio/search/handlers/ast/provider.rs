use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_server::transport::{AstSearchFlightRouteProvider, SearchFlightRouteResponse};

use super::batch::build_ast_hits_flight_batch;
use super::response::load_ast_search_response;
use crate::studio::GatewayState;
use crate::studio::search::handlers::queries::AstSearchQuery;

pub(crate) struct StudioAstSearchFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioAstSearchFlightRouteProvider {
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioAstSearchFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioAstSearchFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AstSearchFlightRouteProvider for StudioAstSearchFlightRouteProvider {
    async fn ast_search_batch(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<SearchFlightRouteResponse, String> {
        let response = load_ast_search_response(
            self.state.as_ref(),
            AstSearchQuery {
                q: Some(query_text.to_string()),
                limit: Some(limit),
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
        build_ast_hits_flight_batch(response.hits.as_slice())
            .map(|batch| SearchFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
    }
}
