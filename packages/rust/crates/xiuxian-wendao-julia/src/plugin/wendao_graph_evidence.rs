use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema};
use xiuxian_wendao_runtime::transport::normalize_flight_route;

/// Default schema version for the Rust mirror of the `WendaoGraph` evidence contract.
pub const WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION: &str = "v0-draft";

/// Planned Flight route for `LinkGraph` evidence requests handled by `WendaoGraph.jl`.
pub const WENDAO_GRAPH_LINK_EVIDENCE_ROUTE: &str = "/graph/link/evidence";

/// Canonical `WendaoGraph` evidence request table names.
pub const WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES: [&str; 5] = [
    "nodes",
    "edges",
    "seeds",
    "semantic_neighbors",
    "semantic_overlay",
];

/// Canonical `WendaoGraph` evidence response table names.
pub const WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES: [&str; 17] = [
    "graph_metrics",
    "components",
    "topology_profile",
    "topology_candidates",
    "topology_bottlenecks",
    "topology_communities",
    "topology_cover",
    "topology_core",
    "topology_boundary",
    "topology_transitions",
    "topology_gateways",
    "topology_community_summaries",
    "topology_community_links",
    "topology_community_frontier",
    "semantic_overlay",
    "diffusion_scores",
    "link_frontier",
];

/// Scalar Arrow type used by a `WendaoGraph` evidence table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoGraphEvidenceColumnType {
    /// UTF-8 string column.
    Utf8,
    /// 64-bit integer column.
    Int64,
    /// 64-bit float column.
    Float64,
    /// Boolean column.
    Boolean,
}

impl WendaoGraphEvidenceColumnType {
    #[must_use]
    fn data_type(self) -> DataType {
        match self {
            Self::Utf8 => DataType::Utf8,
            Self::Int64 => DataType::Int64,
            Self::Float64 => DataType::Float64,
            Self::Boolean => DataType::Boolean,
        }
    }
}

/// One column in a `WendaoGraph` evidence table contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WendaoGraphEvidenceColumnContract {
    /// Canonical column name.
    pub name: &'static str,
    /// Canonical Arrow scalar type.
    pub data_type: WendaoGraphEvidenceColumnType,
}

/// Whether a table belongs to the request or response side of the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoGraphEvidenceTableKind {
    /// Host-to-Julia request table.
    Request,
    /// Julia-to-host response table.
    Response,
}

/// One table in the `WendaoGraph` evidence contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WendaoGraphEvidenceTableContract {
    /// Canonical table name.
    pub table_name: &'static str,
    /// Request or response side.
    pub kind: WendaoGraphEvidenceTableKind,
    /// Whether the table must be present in a request bundle.
    pub required: bool,
    /// Canonical ordered columns.
    pub columns: &'static [WendaoGraphEvidenceColumnContract],
}

impl WendaoGraphEvidenceTableContract {
    /// Materialize the Arrow schema for this table contract.
    #[must_use]
    pub fn schema(self) -> Arc<Schema> {
        Arc::new(Schema::new(
            self.columns
                .iter()
                .map(|column| Field::new(column.name, column.data_type.data_type(), false))
                .collect::<Vec<_>>(),
        ))
    }
}

const NODE_COLUMNS: [WendaoGraphEvidenceColumnContract; 1] =
    [column("node_id", WendaoGraphEvidenceColumnType::Utf8)];
const EDGE_COLUMNS: [WendaoGraphEvidenceColumnContract; 2] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
];
const SEED_COLUMNS: [WendaoGraphEvidenceColumnContract; 2] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("weight", WendaoGraphEvidenceColumnType::Float64),
];
const SEMANTIC_NEIGHBOR_COLUMNS: [WendaoGraphEvidenceColumnContract; 6] = [
    column("query_id", WendaoGraphEvidenceColumnType::Utf8),
    column("neighbor_id", WendaoGraphEvidenceColumnType::Utf8),
    column("query_index", WendaoGraphEvidenceColumnType::Int64),
    column("neighbor_index", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("distance", WendaoGraphEvidenceColumnType::Float64),
];
const SEMANTIC_OVERLAY_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
    column("source_index", WendaoGraphEvidenceColumnType::Int64),
    column("target_index", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("distance", WendaoGraphEvidenceColumnType::Float64),
    column("weight", WendaoGraphEvidenceColumnType::Float64),
    column("edge_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const GRAPH_METRIC_COLUMNS: [WendaoGraphEvidenceColumnContract; 5] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("in_degree", WendaoGraphEvidenceColumnType::Int64),
    column("out_degree", WendaoGraphEvidenceColumnType::Int64),
    column("degree", WendaoGraphEvidenceColumnType::Int64),
];
const COMPONENT_COLUMNS: [WendaoGraphEvidenceColumnContract; 5] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("component_id", WendaoGraphEvidenceColumnType::Int64),
    column("component_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("component_size", WendaoGraphEvidenceColumnType::Int64),
];
const TOPOLOGY_PROFILE_COLUMNS: [WendaoGraphEvidenceColumnContract; 11] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("weak_component_id", WendaoGraphEvidenceColumnType::Int64),
    column("weak_component_size", WendaoGraphEvidenceColumnType::Int64),
    column("strong_component_id", WendaoGraphEvidenceColumnType::Int64),
    column(
        "strong_component_size",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("pagerank_score", WendaoGraphEvidenceColumnType::Float64),
    column("degree_centrality", WendaoGraphEvidenceColumnType::Float64),
    column("topology_prior", WendaoGraphEvidenceColumnType::Float64),
    column("topology_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_CANDIDATE_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("seed_id", WendaoGraphEvidenceColumnType::Utf8),
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("distance", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("topology_score", WendaoGraphEvidenceColumnType::Float64),
    column("topology_prior", WendaoGraphEvidenceColumnType::Float64),
    column("topology_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_BOTTLENECK_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("is_articulation", WendaoGraphEvidenceColumnType::Boolean),
    column(
        "bridge_endpoint_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "biconnected_component_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("bottleneck_score", WendaoGraphEvidenceColumnType::Float64),
    column("bottleneck_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_COMMUNITY_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column("community_size", WendaoGraphEvidenceColumnType::Int64),
    column("community_count", WendaoGraphEvidenceColumnType::Int64),
    column("community_score", WendaoGraphEvidenceColumnType::Float64),
    column("modularity_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_COVER_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("anchor_id", WendaoGraphEvidenceColumnType::Utf8),
    column("anchor_vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("is_anchor", WendaoGraphEvidenceColumnType::Boolean),
    column("cover_distance", WendaoGraphEvidenceColumnType::Int64),
    column("anchor_degree", WendaoGraphEvidenceColumnType::Int64),
    column("cover_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_CORE_COLUMNS: [WendaoGraphEvidenceColumnContract; 7] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("core_number", WendaoGraphEvidenceColumnType::Int64),
    column("max_core_number", WendaoGraphEvidenceColumnType::Int64),
    column("core_score", WendaoGraphEvidenceColumnType::Float64),
    column("core_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_BOUNDARY_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column(
        "internal_neighbor_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "external_neighbor_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("boundary_ratio", WendaoGraphEvidenceColumnType::Float64),
    column("boundary_score", WendaoGraphEvidenceColumnType::Float64),
    column("boundary_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_TRANSITION_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
    column("source_index", WendaoGraphEvidenceColumnType::Int64),
    column("target_index", WendaoGraphEvidenceColumnType::Int64),
    column("source_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("target_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("is_cross_community", WendaoGraphEvidenceColumnType::Boolean),
    column("transition_score", WendaoGraphEvidenceColumnType::Float64),
    column("transition_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_GATEWAY_COLUMNS: [WendaoGraphEvidenceColumnContract; 9] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column(
        "incoming_transition_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "outgoing_transition_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "transition_community_count",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("gateway_score", WendaoGraphEvidenceColumnType::Float64),
    column("gateway_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_COMMUNITY_SUMMARY_COLUMNS: [WendaoGraphEvidenceColumnContract; 11] = [
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column("community_size", WendaoGraphEvidenceColumnType::Int64),
    column("community_count", WendaoGraphEvidenceColumnType::Int64),
    column(
        "representative_node_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_vertex_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("gateway_count", WendaoGraphEvidenceColumnType::Int64),
    column("boundary_count", WendaoGraphEvidenceColumnType::Int64),
    column("transition_count", WendaoGraphEvidenceColumnType::Int64),
    column("summary_score", WendaoGraphEvidenceColumnType::Float64),
    column("summary_role", WendaoGraphEvidenceColumnType::Utf8),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_COMMUNITY_LINK_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("source_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("target_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("transition_count", WendaoGraphEvidenceColumnType::Int64),
    column(
        "representative_source_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_target_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_source_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "representative_target_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column(
        "representative_transition_score",
        WendaoGraphEvidenceColumnType::Float64,
    ),
    column("link_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
];
const TOPOLOGY_COMMUNITY_FRONTIER_COLUMNS: [WendaoGraphEvidenceColumnContract; 15] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_community_id", WendaoGraphEvidenceColumnType::Int64),
    column("community_id", WendaoGraphEvidenceColumnType::Int64),
    column("depth", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column(
        "representative_node_id",
        WendaoGraphEvidenceColumnType::Utf8,
    ),
    column(
        "representative_vertex_index",
        WendaoGraphEvidenceColumnType::Int64,
    ),
    column("community_score", WendaoGraphEvidenceColumnType::Float64),
    column("link_score", WendaoGraphEvidenceColumnType::Float64),
    column("path_score", WendaoGraphEvidenceColumnType::Float64),
    column("transition_count", WendaoGraphEvidenceColumnType::Int64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("disclosure_budget", WendaoGraphEvidenceColumnType::Int64),
];
const DIFFUSION_SCORE_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("diffusion_score", WendaoGraphEvidenceColumnType::Float64),
    column("seed_score", WendaoGraphEvidenceColumnType::Float64),
    column("link_score", WendaoGraphEvidenceColumnType::Float64),
    column("semantic_score", WendaoGraphEvidenceColumnType::Float64),
    column("iteration_count", WendaoGraphEvidenceColumnType::Int64),
    column("residual", WendaoGraphEvidenceColumnType::Float64),
];
const LINK_FRONTIER_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_id", WendaoGraphEvidenceColumnType::Utf8),
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("vertex_index", WendaoGraphEvidenceColumnType::Int64),
    column("depth", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("diffusion_score", WendaoGraphEvidenceColumnType::Float64),
    column("path_score", WendaoGraphEvidenceColumnType::Float64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("disclosure_budget", WendaoGraphEvidenceColumnType::Int64),
];

/// Canonical `WendaoGraph` request table contracts.
pub const WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS: [WendaoGraphEvidenceTableContract; 5] = [
    request_table("nodes", true, &NODE_COLUMNS),
    request_table("edges", true, &EDGE_COLUMNS),
    request_table("seeds", false, &SEED_COLUMNS),
    request_table("semantic_neighbors", false, &SEMANTIC_NEIGHBOR_COLUMNS),
    request_table("semantic_overlay", false, &SEMANTIC_OVERLAY_COLUMNS),
];

/// Canonical `WendaoGraph` response table contracts.
pub const WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS: [WendaoGraphEvidenceTableContract; 17] = [
    response_table("graph_metrics", &GRAPH_METRIC_COLUMNS),
    response_table("components", &COMPONENT_COLUMNS),
    response_table("topology_profile", &TOPOLOGY_PROFILE_COLUMNS),
    response_table("topology_candidates", &TOPOLOGY_CANDIDATE_COLUMNS),
    response_table("topology_bottlenecks", &TOPOLOGY_BOTTLENECK_COLUMNS),
    response_table("topology_communities", &TOPOLOGY_COMMUNITY_COLUMNS),
    response_table("topology_cover", &TOPOLOGY_COVER_COLUMNS),
    response_table("topology_core", &TOPOLOGY_CORE_COLUMNS),
    response_table("topology_boundary", &TOPOLOGY_BOUNDARY_COLUMNS),
    response_table("topology_transitions", &TOPOLOGY_TRANSITION_COLUMNS),
    response_table("topology_gateways", &TOPOLOGY_GATEWAY_COLUMNS),
    response_table(
        "topology_community_summaries",
        &TOPOLOGY_COMMUNITY_SUMMARY_COLUMNS,
    ),
    response_table("topology_community_links", &TOPOLOGY_COMMUNITY_LINK_COLUMNS),
    response_table(
        "topology_community_frontier",
        &TOPOLOGY_COMMUNITY_FRONTIER_COLUMNS,
    ),
    response_table("semantic_overlay", &SEMANTIC_OVERLAY_COLUMNS),
    response_table("diffusion_scores", &DIFFUSION_SCORE_COLUMNS),
    response_table("link_frontier", &LINK_FRONTIER_COLUMNS),
];

const fn column(
    name: &'static str,
    data_type: WendaoGraphEvidenceColumnType,
) -> WendaoGraphEvidenceColumnContract {
    WendaoGraphEvidenceColumnContract { name, data_type }
}

const fn request_table(
    table_name: &'static str,
    required: bool,
    columns: &'static [WendaoGraphEvidenceColumnContract],
) -> WendaoGraphEvidenceTableContract {
    WendaoGraphEvidenceTableContract {
        table_name,
        kind: WendaoGraphEvidenceTableKind::Request,
        required,
        columns,
    }
}

const fn response_table(
    table_name: &'static str,
    columns: &'static [WendaoGraphEvidenceColumnContract],
) -> WendaoGraphEvidenceTableContract {
    WendaoGraphEvidenceTableContract {
        table_name,
        kind: WendaoGraphEvidenceTableKind::Response,
        required: true,
        columns,
    }
}

/// Resolve one route into the planned `WendaoGraph` `LinkGraph` evidence route.
///
/// # Errors
///
/// Returns an error when the route does not normalize to the planned
/// `WendaoGraph` evidence path.
pub fn wendao_graph_link_evidence_route(route: impl AsRef<str>) -> Result<&'static str, String> {
    let normalized = normalize_flight_route(route)?;
    if normalized == WENDAO_GRAPH_LINK_EVIDENCE_ROUTE {
        Ok(WENDAO_GRAPH_LINK_EVIDENCE_ROUTE)
    } else {
        Err(format!(
            "unsupported WendaoGraph evidence Flight route `{normalized}`"
        ))
    }
}

/// Return whether one route belongs to the `WendaoGraph` evidence contract.
#[must_use]
pub fn is_wendao_graph_link_evidence_route(route: impl AsRef<str>) -> bool {
    wendao_graph_link_evidence_route(route).is_ok()
}

/// Resolve one request table contract by table name.
///
/// # Errors
///
/// Returns an error when the table name is not part of the canonical
/// `WendaoGraph` evidence request contract.
pub fn wendao_graph_evidence_request_table_contract(
    table_name: impl AsRef<str>,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    find_contract(
        table_name.as_ref(),
        &WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS,
        "request",
    )
}

/// Resolve one response table contract by table name.
///
/// # Errors
///
/// Returns an error when the table name is not part of the canonical
/// `WendaoGraph` evidence response contract.
pub fn wendao_graph_evidence_response_table_contract(
    table_name: impl AsRef<str>,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    find_contract(
        table_name.as_ref(),
        &WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS,
        "response",
    )
}

/// Materialize the Arrow schema for one `WendaoGraph` evidence table.
///
/// # Errors
///
/// Returns an error when the table is unknown for the selected side.
pub fn wendao_graph_evidence_table_schema(
    kind: WendaoGraphEvidenceTableKind,
    table_name: impl AsRef<str>,
) -> Result<Arc<Schema>, String> {
    let contract = match kind {
        WendaoGraphEvidenceTableKind::Request => {
            wendao_graph_evidence_request_table_contract(table_name)?
        }
        WendaoGraphEvidenceTableKind::Response => {
            wendao_graph_evidence_response_table_contract(table_name)?
        }
    };
    Ok(contract.schema())
}

/// Validate a request table Arrow schema against the canonical contract.
///
/// # Errors
///
/// Returns an error when the table name is unknown or the schema order, column
/// type, or nullability does not match the canonical request contract.
pub fn validate_wendao_graph_evidence_request_schema(
    table_name: impl AsRef<str>,
    schema: &Schema,
) -> Result<(), String> {
    let contract = wendao_graph_evidence_request_table_contract(table_name)?;
    validate_contract_schema(contract, schema)
}

/// Validate a response table Arrow schema against the canonical contract.
///
/// # Errors
///
/// Returns an error when the table name is unknown or the schema order, column
/// type, or nullability does not match the canonical response contract.
pub fn validate_wendao_graph_evidence_response_schema(
    table_name: impl AsRef<str>,
    schema: &Schema,
) -> Result<(), String> {
    let contract = wendao_graph_evidence_response_table_contract(table_name)?;
    validate_contract_schema(contract, schema)
}

fn find_contract(
    table_name: &str,
    contracts: &'static [WendaoGraphEvidenceTableContract],
    side: &str,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    contracts
        .iter()
        .find(|contract| contract.table_name == table_name)
        .ok_or_else(|| format!("unknown WendaoGraph evidence {side} table `{table_name}`"))
}

fn validate_contract_schema(
    contract: &WendaoGraphEvidenceTableContract,
    schema: &Schema,
) -> Result<(), String> {
    if schema.fields().len() != contract.columns.len() {
        return Err(format!(
            "WendaoGraph evidence table `{}` must have {} columns, got {}",
            contract.table_name,
            contract.columns.len(),
            schema.fields().len()
        ));
    }
    for (index, column) in contract.columns.iter().enumerate() {
        let field = schema.field(index);
        let expected_type = column.data_type.data_type();
        if field.name() != column.name {
            return Err(format!(
                "WendaoGraph evidence table `{}` column {} must be `{}`, got `{}`",
                contract.table_name,
                index,
                column.name,
                field.name()
            ));
        }
        if field.data_type() != &expected_type {
            return Err(format!(
                "WendaoGraph evidence table `{}` column `{}` must be {:?}, got {:?}",
                contract.table_name,
                column.name,
                expected_type,
                field.data_type()
            ));
        }
        if field.is_nullable() {
            return Err(format!(
                "WendaoGraph evidence table `{}` column `{}` must be non-nullable",
                contract.table_name, column.name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/wendao_graph_evidence.rs"]
mod tests;
