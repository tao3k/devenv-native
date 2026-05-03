use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceArrayRef, LanceBooleanArray, LanceDataType, LanceField, LanceInt32Array, LanceRecordBatch,
    LanceSchema, LanceStringArray,
};
use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao_web::transport::{
    GraphNeighborsFlightRouteProvider, GraphNeighborsFlightRouteResponse,
};

use super::service::run_graph_neighbors;
use super::shared::{normalize_hops, normalize_limit, parse_direction};
use crate::studio::router::{GatewayState, StudioApiError};
use crate::studio::types::{GraphLink, GraphNeighborsResponse, GraphNode};

/// Studio-backed Flight provider for the semantic `/graph/neighbors` route.
#[derive(Clone)]
pub(crate) struct StudioGraphNeighborsFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioGraphNeighborsFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioGraphNeighborsFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioGraphNeighborsFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl GraphNeighborsFlightRouteProvider for StudioGraphNeighborsFlightRouteProvider {
    async fn graph_neighbors_batch(
        &self,
        node_id: &str,
        direction: &str,
        hops: usize,
        limit: usize,
    ) -> Result<GraphNeighborsFlightRouteResponse, Status> {
        load_graph_neighbors_flight_response(
            Arc::clone(&self.state),
            node_id,
            direction,
            hops,
            limit,
        )
        .await
        .map_err(studio_api_error_to_tonic_status)
    }
}

pub(crate) async fn build_graph_neighbors_response(
    state: Arc<GatewayState>,
    node_id: &str,
    direction: &str,
    hops: usize,
    limit: usize,
) -> Result<GraphNeighborsResponse, StudioApiError> {
    let node_id = node_id.trim();
    if node_id.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_NODE_ID",
            "`nodeId` is required",
        ));
    }
    run_graph_neighbors(
        state,
        node_id,
        parse_direction(Some(direction)),
        normalize_hops(Some(hops)),
        normalize_limit(Some(limit)),
    )
    .await
}

pub(crate) async fn load_graph_neighbors_flight_response(
    state: Arc<GatewayState>,
    node_id: &str,
    direction: &str,
    hops: usize,
    limit: usize,
) -> Result<GraphNeighborsFlightRouteResponse, StudioApiError> {
    let response = build_graph_neighbors_response(state, node_id, direction, hops, limit).await?;
    let batch = graph_neighbors_response_batch(&response).map_err(|error| {
        StudioApiError::internal(
            "GRAPH_NEIGHBORS_FLIGHT_BATCH_FAILED",
            "Failed to materialize graph neighbors through the Flight-backed provider",
            Some(error),
        )
    })?;
    Ok(GraphNeighborsFlightRouteResponse::new(batch))
}

pub(crate) fn graph_neighbors_response_batch(
    response: &GraphNeighborsResponse,
) -> Result<LanceRecordBatch, String> {
    let rows = graph_neighbors_rows(response);
    let columns = graph_neighbors_response_columns(&rows)?;
    LanceRecordBatch::try_new(graph_neighbors_response_schema(), columns)
        .map_err(|error| format!("failed to build graph-neighbors Flight batch: {error}"))
}

fn graph_neighbors_rows(response: &GraphNeighborsResponse) -> Vec<FlightGraphRow> {
    let node_rows = response.nodes.iter().map(FlightGraphRow::from_node);
    let link_rows = response.links.iter().map(FlightGraphRow::from_link);
    node_rows.chain(link_rows).collect()
}

fn graph_neighbors_response_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new("rowType", LanceDataType::Utf8, false),
        LanceField::new("nodeId", LanceDataType::Utf8, true),
        LanceField::new("nodeLabel", LanceDataType::Utf8, true),
        LanceField::new("nodePath", LanceDataType::Utf8, true),
        LanceField::new("nodeType", LanceDataType::Utf8, true),
        LanceField::new("nodeIsCenter", LanceDataType::Boolean, true),
        LanceField::new("nodeDistance", LanceDataType::Int32, true),
        LanceField::new("navigationPath", LanceDataType::Utf8, true),
        LanceField::new("navigationCategory", LanceDataType::Utf8, true),
        LanceField::new("navigationProjectName", LanceDataType::Utf8, true),
        LanceField::new("navigationRootLabel", LanceDataType::Utf8, true),
        LanceField::new("navigationLine", LanceDataType::Int32, true),
        LanceField::new("navigationLineEnd", LanceDataType::Int32, true),
        LanceField::new("navigationColumn", LanceDataType::Int32, true),
        LanceField::new("linkSource", LanceDataType::Utf8, true),
        LanceField::new("linkTarget", LanceDataType::Utf8, true),
        LanceField::new("linkDirection", LanceDataType::Utf8, true),
        LanceField::new("linkDistance", LanceDataType::Int32, true),
    ]))
}

fn graph_neighbors_response_columns(rows: &[FlightGraphRow]) -> Result<Vec<LanceArrayRef>, String> {
    Ok(vec![
        Arc::new(LanceStringArray::from(
            rows.iter().map(|row| row.row_type).collect::<Vec<_>>(),
        )),
        graph_neighbors_string_column(rows, |row| row.node_id.as_deref()),
        graph_neighbors_string_column(rows, |row| row.node_label.as_deref()),
        graph_neighbors_string_column(rows, |row| row.node_path.as_deref()),
        graph_neighbors_string_column(rows, |row| row.node_type.as_deref()),
        Arc::new(LanceBooleanArray::from(
            rows.iter()
                .map(|row| row.node_is_center)
                .collect::<Vec<_>>(),
        )),
        graph_neighbors_int32_column(rows, |row| row.node_distance)?,
        graph_neighbors_string_column(rows, |row| row.navigation_path.as_deref()),
        graph_neighbors_string_column(rows, |row| row.navigation_category.as_deref()),
        graph_neighbors_string_column(rows, |row| row.navigation_project_name.as_deref()),
        graph_neighbors_string_column(rows, |row| row.navigation_root_label.as_deref()),
        graph_neighbors_int32_column(rows, |row| row.navigation_line)?,
        graph_neighbors_int32_column(rows, |row| row.navigation_line_end)?,
        graph_neighbors_int32_column(rows, |row| row.navigation_column)?,
        graph_neighbors_string_column(rows, |row| row.link_source.as_deref()),
        graph_neighbors_string_column(rows, |row| row.link_target.as_deref()),
        graph_neighbors_string_column(rows, |row| row.link_direction.as_deref()),
        graph_neighbors_int32_column(rows, |row| row.link_distance)?,
    ])
}

fn graph_neighbors_string_column<F>(rows: &[FlightGraphRow], accessor: F) -> LanceArrayRef
where
    F: Fn(&FlightGraphRow) -> Option<&str>,
{
    Arc::new(LanceStringArray::from(
        rows.iter().map(accessor).collect::<Vec<_>>(),
    ))
}

fn graph_neighbors_int32_column<F>(
    rows: &[FlightGraphRow],
    accessor: F,
) -> Result<LanceArrayRef, String>
where
    F: Fn(&FlightGraphRow) -> Option<usize>,
{
    Ok(Arc::new(LanceInt32Array::from(
        rows.iter()
            .map(|row| accessor(row).map(usize_to_i32).transpose())
            .collect::<Result<Vec<_>, _>>()?,
    )))
}

#[derive(Debug, Clone)]
struct FlightGraphRow {
    row_type: &'static str,
    node_id: Option<String>,
    node_label: Option<String>,
    node_path: Option<String>,
    node_type: Option<String>,
    node_is_center: Option<bool>,
    node_distance: Option<usize>,
    navigation_path: Option<String>,
    navigation_category: Option<String>,
    navigation_project_name: Option<String>,
    navigation_root_label: Option<String>,
    navigation_line: Option<usize>,
    navigation_line_end: Option<usize>,
    navigation_column: Option<usize>,
    link_source: Option<String>,
    link_target: Option<String>,
    link_direction: Option<String>,
    link_distance: Option<usize>,
}

impl FlightGraphRow {
    fn from_node(node: &GraphNode) -> Self {
        Self {
            row_type: "node",
            node_id: Some(node.id.clone()),
            node_label: Some(node.label.clone()),
            node_path: Some(node.path.clone()),
            node_type: Some(node.node_type.clone()),
            node_is_center: Some(node.is_center),
            node_distance: Some(node.distance),
            navigation_path: node
                .navigation_target
                .as_ref()
                .map(|target| target.path.clone()),
            navigation_category: node
                .navigation_target
                .as_ref()
                .map(|target| target.category.clone()),
            navigation_project_name: node
                .navigation_target
                .as_ref()
                .and_then(|target| target.project_name.clone()),
            navigation_root_label: node
                .navigation_target
                .as_ref()
                .and_then(|target| target.root_label.clone()),
            navigation_line: node
                .navigation_target
                .as_ref()
                .and_then(|target| target.line),
            navigation_line_end: node
                .navigation_target
                .as_ref()
                .and_then(|target| target.line_end),
            navigation_column: node
                .navigation_target
                .as_ref()
                .and_then(|target| target.column),
            link_source: None,
            link_target: None,
            link_direction: None,
            link_distance: None,
        }
    }

    fn from_link(link: &GraphLink) -> Self {
        Self {
            row_type: "link",
            node_id: None,
            node_label: None,
            node_path: None,
            node_type: None,
            node_is_center: None,
            node_distance: None,
            navigation_path: None,
            navigation_category: None,
            navigation_project_name: None,
            navigation_root_label: None,
            navigation_line: None,
            navigation_line_end: None,
            navigation_column: None,
            link_source: Some(link.source.clone()),
            link_target: Some(link.target.clone()),
            link_direction: Some(link.direction.clone()),
            link_distance: Some(link.distance),
        }
    }
}

fn usize_to_i32(value: usize) -> Result<i32, String> {
    i32::try_from(value)
        .map_err(|error| format!("failed to represent graph-neighbors position: {error}"))
}

fn studio_api_error_to_tonic_status(error: StudioApiError) -> Status {
    match error.status() {
        axum::http::StatusCode::BAD_REQUEST => Status::invalid_argument(error.error.message),
        axum::http::StatusCode::NOT_FOUND => Status::not_found(error.error.message),
        axum::http::StatusCode::CONFLICT => Status::failed_precondition(error.error.message),
        _ => Status::internal(error.error.message),
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/graph/flight.rs"]
mod tests;
