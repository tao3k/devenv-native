use std::sync::Arc;

use async_trait::async_trait;
use tonic::Status;
use xiuxian_db_store::{
    LanceArrayRef, LanceDataType, LanceField, LanceFloat32Array, LanceInt32Array, LanceRecordBatch,
    LanceSchema, LanceStringArray,
};
use xiuxian_wendao_runtime::transport::{
    Topology3dFlightRouteProvider, Topology3dFlightRouteResponse,
};

use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::gateway::studio::types::{
    Topology3dPayload, TopologyCluster, TopologyLink, TopologyNode,
};

use super::service::run_topology_3d;

/// Studio-backed Flight provider for the semantic `/topology/3d` route.
#[derive(Clone)]
pub(crate) struct StudioTopology3dFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioTopology3dFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioTopology3dFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioTopology3dFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl Topology3dFlightRouteProvider for StudioTopology3dFlightRouteProvider {
    async fn topology_3d_batch(&self) -> Result<Topology3dFlightRouteResponse, Status> {
        load_topology_3d_flight_response(Arc::clone(&self.state))
            .await
            .map_err(studio_api_error_to_tonic_status)
    }
}

pub(crate) async fn load_topology_3d_flight_response(
    state: Arc<GatewayState>,
) -> Result<Topology3dFlightRouteResponse, StudioApiError> {
    let response = run_topology_3d(state).await?;
    let batch = topology_3d_response_batch(&response).map_err(|error| {
        StudioApiError::internal(
            "TOPOLOGY_3D_FLIGHT_BATCH_FAILED",
            "Failed to materialize topology through the Flight-backed provider",
            Some(error),
        )
    })?;
    Ok(Topology3dFlightRouteResponse::new(batch))
}

pub(crate) fn topology_3d_response_batch(
    response: &Topology3dPayload,
) -> Result<LanceRecordBatch, String> {
    let rows = topology_3d_rows(response);
    let columns = topology_3d_response_columns(&rows)?;
    LanceRecordBatch::try_new(topology_3d_response_schema(), columns)
        .map_err(|error| format!("failed to build topology-3d Flight batch: {error}"))
}

fn topology_3d_rows(response: &Topology3dPayload) -> Vec<FlightTopologyRow> {
    let node_rows = response.nodes.iter().map(FlightTopologyRow::from_node);
    let link_rows = response.links.iter().map(FlightTopologyRow::from_link);
    let cluster_rows = response
        .clusters
        .iter()
        .map(FlightTopologyRow::from_cluster);
    node_rows.chain(link_rows).chain(cluster_rows).collect()
}

fn topology_3d_response_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new("rowType", LanceDataType::Utf8, false),
        LanceField::new("nodeId", LanceDataType::Utf8, true),
        LanceField::new("nodeName", LanceDataType::Utf8, true),
        LanceField::new("nodeType", LanceDataType::Utf8, true),
        LanceField::new("nodePosX", LanceDataType::Float32, true),
        LanceField::new("nodePosY", LanceDataType::Float32, true),
        LanceField::new("nodePosZ", LanceDataType::Float32, true),
        LanceField::new("nodeClusterId", LanceDataType::Utf8, true),
        LanceField::new("linkFrom", LanceDataType::Utf8, true),
        LanceField::new("linkTo", LanceDataType::Utf8, true),
        LanceField::new("linkLabel", LanceDataType::Utf8, true),
        LanceField::new("clusterId", LanceDataType::Utf8, true),
        LanceField::new("clusterName", LanceDataType::Utf8, true),
        LanceField::new("clusterCentroidX", LanceDataType::Float32, true),
        LanceField::new("clusterCentroidY", LanceDataType::Float32, true),
        LanceField::new("clusterCentroidZ", LanceDataType::Float32, true),
        LanceField::new("clusterNodeCount", LanceDataType::Int32, true),
        LanceField::new("clusterColor", LanceDataType::Utf8, true),
    ]))
}

fn topology_3d_response_columns(rows: &[FlightTopologyRow]) -> Result<Vec<LanceArrayRef>, String> {
    Ok(vec![
        Arc::new(LanceStringArray::from(
            rows.iter().map(|row| row.row_type).collect::<Vec<_>>(),
        )),
        topology_string_column(rows, |row| row.node_id.as_deref()),
        topology_string_column(rows, |row| row.node_name.as_deref()),
        topology_string_column(rows, |row| row.node_type.as_deref()),
        topology_float32_column(rows, |row| row.node_pos_x),
        topology_float32_column(rows, |row| row.node_pos_y),
        topology_float32_column(rows, |row| row.node_pos_z),
        topology_string_column(rows, |row| row.node_cluster_id.as_deref()),
        topology_string_column(rows, |row| row.link_from.as_deref()),
        topology_string_column(rows, |row| row.link_to.as_deref()),
        topology_string_column(rows, |row| row.link_label.as_deref()),
        topology_string_column(rows, |row| row.cluster_id.as_deref()),
        topology_string_column(rows, |row| row.cluster_name.as_deref()),
        topology_float32_column(rows, |row| row.cluster_centroid_x),
        topology_float32_column(rows, |row| row.cluster_centroid_y),
        topology_float32_column(rows, |row| row.cluster_centroid_z),
        topology_int32_column(rows, |row| row.cluster_node_count)?,
        topology_string_column(rows, |row| row.cluster_color.as_deref()),
    ])
}

fn topology_string_column<F>(rows: &[FlightTopologyRow], accessor: F) -> LanceArrayRef
where
    F: Fn(&FlightTopologyRow) -> Option<&str>,
{
    Arc::new(LanceStringArray::from(
        rows.iter().map(accessor).collect::<Vec<_>>(),
    ))
}

fn topology_float32_column<F>(rows: &[FlightTopologyRow], accessor: F) -> LanceArrayRef
where
    F: Fn(&FlightTopologyRow) -> Option<f32>,
{
    Arc::new(LanceFloat32Array::from(
        rows.iter().map(accessor).collect::<Vec<_>>(),
    ))
}

fn topology_int32_column<F>(
    rows: &[FlightTopologyRow],
    accessor: F,
) -> Result<LanceArrayRef, String>
where
    F: Fn(&FlightTopologyRow) -> Option<usize>,
{
    Ok(Arc::new(LanceInt32Array::from(
        rows.iter()
            .map(|row| accessor(row).map(usize_to_i32).transpose())
            .collect::<Result<Vec<_>, _>>()?,
    )))
}

#[derive(Debug, Clone)]
struct FlightTopologyRow {
    row_type: &'static str,
    node_id: Option<String>,
    node_name: Option<String>,
    node_type: Option<String>,
    node_pos_x: Option<f32>,
    node_pos_y: Option<f32>,
    node_pos_z: Option<f32>,
    node_cluster_id: Option<String>,
    link_from: Option<String>,
    link_to: Option<String>,
    link_label: Option<String>,
    cluster_id: Option<String>,
    cluster_name: Option<String>,
    cluster_centroid_x: Option<f32>,
    cluster_centroid_y: Option<f32>,
    cluster_centroid_z: Option<f32>,
    cluster_node_count: Option<usize>,
    cluster_color: Option<String>,
}

impl FlightTopologyRow {
    fn from_node(node: &TopologyNode) -> Self {
        Self {
            row_type: "node",
            node_id: Some(node.id.clone()),
            node_name: Some(node.name.clone()),
            node_type: Some(node.node_type.clone()),
            node_pos_x: Some(node.position[0]),
            node_pos_y: Some(node.position[1]),
            node_pos_z: Some(node.position[2]),
            node_cluster_id: node.cluster_id.clone(),
            link_from: None,
            link_to: None,
            link_label: None,
            cluster_id: None,
            cluster_name: None,
            cluster_centroid_x: None,
            cluster_centroid_y: None,
            cluster_centroid_z: None,
            cluster_node_count: None,
            cluster_color: None,
        }
    }

    fn from_link(link: &TopologyLink) -> Self {
        Self {
            row_type: "link",
            node_id: None,
            node_name: None,
            node_type: None,
            node_pos_x: None,
            node_pos_y: None,
            node_pos_z: None,
            node_cluster_id: None,
            link_from: Some(link.from.clone()),
            link_to: Some(link.to.clone()),
            link_label: link.label.clone(),
            cluster_id: None,
            cluster_name: None,
            cluster_centroid_x: None,
            cluster_centroid_y: None,
            cluster_centroid_z: None,
            cluster_node_count: None,
            cluster_color: None,
        }
    }

    fn from_cluster(cluster: &TopologyCluster) -> Self {
        Self {
            row_type: "cluster",
            node_id: None,
            node_name: None,
            node_type: None,
            node_pos_x: None,
            node_pos_y: None,
            node_pos_z: None,
            node_cluster_id: None,
            link_from: None,
            link_to: None,
            link_label: None,
            cluster_id: Some(cluster.id.clone()),
            cluster_name: Some(cluster.name.clone()),
            cluster_centroid_x: Some(cluster.centroid[0]),
            cluster_centroid_y: Some(cluster.centroid[1]),
            cluster_centroid_z: Some(cluster.centroid[2]),
            cluster_node_count: Some(cluster.node_count),
            cluster_color: Some(cluster.color.clone()),
        }
    }
}

fn usize_to_i32(value: usize) -> Result<i32, String> {
    i32::try_from(value).map_err(|error| format!("failed to represent topology row: {error}"))
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
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/graph/topology_flight.rs"]
mod tests;
