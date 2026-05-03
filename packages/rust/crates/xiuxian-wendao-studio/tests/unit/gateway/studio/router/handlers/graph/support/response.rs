use std::sync::Arc;

use axum::extract::{Path as AxumPath, Query, State};

use crate::studio::router::handlers::graph::neighbors::graph_neighbors;
use crate::studio::router::handlers::graph::shared::GraphNeighborsQuery;
use xiuxian_wendao::search::contracts::GraphNeighborsResponse;

use super::fixture::Fixture;

pub(crate) async fn graph_neighbors_response(
    fixture: &Fixture,
    node_id: &str,
    hops: usize,
    limit: usize,
) -> GraphNeighborsResponse {
    graph_neighbors(
        State(Arc::clone(&fixture.state)),
        AxumPath(node_id.to_string()),
        Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(hops),
            limit: Some(limit),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("graph neighbors should succeed: {error:?}"))
    .0
}
