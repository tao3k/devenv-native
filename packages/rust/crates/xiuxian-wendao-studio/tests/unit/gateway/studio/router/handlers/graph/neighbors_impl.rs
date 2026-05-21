//! Test-only graph-neighbor HTTP adapter for retired outward REST routes.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};

use crate::contracts::GraphNeighborsResponse;
use crate::studio::router::handlers::graph::query_support::{
    GraphNeighborsQuery, normalize_hops, normalize_limit, parse_direction,
};
use crate::studio::router::handlers::graph::service::run_graph_neighbors;
use crate::studio::router::{GatewayState, StudioApiError};

pub(crate) async fn graph_neighbors(
    State(state): State<Arc<GatewayState>>,
    AxumPath(node_id): AxumPath<String>,
    Query(query): Query<GraphNeighborsQuery>,
) -> Result<Json<GraphNeighborsResponse>, StudioApiError> {
    let direction = parse_direction(query.direction.as_deref());
    let hops = normalize_hops(query.hops);
    let limit = normalize_limit(query.limit);
    Ok(Json(
        run_graph_neighbors(Arc::clone(&state), node_id.as_str(), direction, hops, limit).await?,
    ))
}
