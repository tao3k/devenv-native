use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_runtime::transport::{
    SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE, SEARCH_REFERENCES_ROUTE, SEARCH_SYMBOLS_ROUTE,
    SearchFlightRouteProvider, SearchFlightRouteResponse,
};

use crate::gateway::studio::router::GatewayState;
use crate::gateway::studio::search::handlers::knowledge::load_intent_search_flight_response;
use crate::gateway::studio::search::handlers::knowledge::load_knowledge_search_flight_response;
use crate::gateway::studio::search::handlers::queries::{ReferenceSearchQuery, SymbolSearchQuery};
use crate::gateway::studio::search::handlers::references::load_reference_search_flight_response;
use crate::gateway::studio::search::handlers::symbols::load_symbol_search_flight_response;

/// Studio-backed aggregate Flight provider for the currently-aligned semantic
/// search families.
#[derive(Clone)]
pub(crate) struct StudioSearchFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioSearchFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioSearchFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioSearchFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl SearchFlightRouteProvider for StudioSearchFlightRouteProvider {
    async fn search_batch(
        &self,
        route: &str,
        query_text: &str,
        limit: usize,
        intent: Option<&str>,
        repo_hint: Option<&str>,
    ) -> Result<SearchFlightRouteResponse, String> {
        match route {
            SEARCH_INTENT_ROUTE => load_intent_search_flight_response(
                Arc::clone(&self.state.studio),
                query_text,
                query_text,
                repo_hint,
                limit,
                intent.map(ToString::to_string),
            )
            .await
            .map_err(|error| {
                format!(
                    "studio aggregate Flight provider failed to build intent response for `{query_text}`: {error:?}"
                )
            }),
            SEARCH_KNOWLEDGE_ROUTE => load_knowledge_search_flight_response(
                Arc::clone(&self.state.studio),
                query_text,
                limit,
            )
            .await
            .map_err(|error| {
                format!(
                    "studio aggregate Flight provider failed to build knowledge response for `{query_text}`: {error:?}"
                )
            }),
            SEARCH_REFERENCES_ROUTE => load_reference_search_flight_response(
                Arc::clone(&self.state),
                ReferenceSearchQuery {
                    q: Some(query_text.to_string()),
                    limit: Some(limit),
                },
            )
            .await
            .map_err(|error| {
                format!(
                    "studio aggregate Flight provider failed to build reference response for `{query_text}`: {error:?}"
                )
            }),
            SEARCH_SYMBOLS_ROUTE => load_symbol_search_flight_response(
                self.state.as_ref(),
                SymbolSearchQuery {
                    q: Some(query_text.to_string()),
                    limit: Some(limit),
                },
            )
            .await
            .map_err(|error| {
                format!(
                    "studio aggregate Flight provider failed to build symbol response for `{query_text}`: {error:?}"
                )
            }),
            _ => Err(format!(
                "studio aggregate Flight provider does not support route `{route}`"
            )),
        }
    }
}
