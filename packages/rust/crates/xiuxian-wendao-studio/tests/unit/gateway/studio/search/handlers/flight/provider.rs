use std::sync::Arc;

use crate::transport::{SEARCH_KNOWLEDGE_ROUTE, SEARCH_SYMBOLS_ROUTE, SearchFlightRouteProvider};

use crate::studio::search::handlers::flight::provider::StudioSearchFlightRouteProvider;
use crate::studio::{GatewayState, StudioState};

use super::{first_string, make_gateway_state_with_search_routes};

#[tokio::test]
async fn studio_search_flight_provider_dispatches_symbol_route() {
    let fixture = make_gateway_state_with_search_routes().await;
    let provider = StudioSearchFlightRouteProvider::new(Arc::clone(&fixture.state));

    let batch = provider
        .search_batch(SEARCH_SYMBOLS_ROUTE, "alpha", 5, None, None)
        .await
        .unwrap_or_else(|error| panic!("symbol route should succeed: {error}"));

    assert!(batch.batch.num_rows() >= 2);
    assert_eq!(first_string(&batch.batch, "name"), "AlphaService");
}

#[tokio::test]
async fn studio_search_flight_provider_dispatches_knowledge_route() {
    let fixture = make_gateway_state_with_search_routes().await;
    let provider = StudioSearchFlightRouteProvider::new(Arc::clone(&fixture.state));

    let batch = provider
        .search_batch(SEARCH_KNOWLEDGE_ROUTE, "alpha", 5, None, None)
        .await
        .unwrap_or_else(|error| panic!("knowledge route should succeed: {error}"));

    assert!(batch.batch.num_rows() >= 1);
    assert_eq!(first_string(&batch.batch, "stem"), "alpha");
    let app_metadata: serde_json::Value = serde_json::from_slice(&batch.app_metadata)
        .unwrap_or_else(|error| panic!("knowledge app_metadata should decode: {error}"));
    assert_eq!(app_metadata["query"], "alpha");
    assert_eq!(app_metadata["hitCount"], 1);
}

#[tokio::test]
async fn studio_search_flight_provider_rejects_unknown_routes() {
    let registry = xiuxian_wendao::analyzers::bootstrap_builtin_registry()
        .unwrap_or_else(|error| panic!("bootstrap registry: {error}"));
    let provider = StudioSearchFlightRouteProvider::new(Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new_with_bootstrap_ui_config(Arc::new(
            registry,
        ))),
    }));

    let Err(error) = provider
        .search_batch("/search/unknown", "alpha", 5, None, None)
        .await
    else {
        panic!("unknown route should be rejected");
    };

    assert!(
        error.contains("/search/unknown"),
        "unexpected error: {error}"
    );
}
