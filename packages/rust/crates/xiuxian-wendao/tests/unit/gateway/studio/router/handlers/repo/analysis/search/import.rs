use std::sync::Arc;

use axum::extract::{Query, State};

use crate::gateway::studio::router::handlers::repo::analysis::search::import::import_search;
use crate::gateway::studio::router::handlers::repo::query::analysis::RepoImportSearchApiQuery;
use crate::gateway::studio::router::{GatewayState, StudioState};

#[tokio::test]
async fn import_search_requires_repo() {
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    let Err(error) = import_search(
        Query(RepoImportSearchApiQuery {
            repo: None,
            package: Some("SciMLBase".to_string()),
            module: None,
            limit: Some(10),
        }),
        State(state),
    )
    .await
    else {
        panic!("missing repo should fail before repository execution");
    };

    assert_eq!(error.code(), "MISSING_REPO");
}

#[tokio::test]
async fn import_search_requires_package_or_module() {
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    let Err(error) = import_search(
        Query(RepoImportSearchApiQuery {
            repo: Some("alpha/repo".to_string()),
            package: None,
            module: None,
            limit: Some(10),
        }),
        State(state),
    )
    .await
    else {
        panic!("missing import filters should fail before repository execution");
    };

    assert_eq!(error.code(), "MISSING_IMPORT_FILTER");
}
