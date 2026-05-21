//! Column constants for the Julia graph-structural Arrow contract.

//! Column constants for the `WendaoSearch.jl` graph-structural Arrow contract.

/// Default schema version for the staged Julia graph-structural contract.
pub const JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION: &str = "v0-draft";

/// Stable route for the structural-rerank graph-search exchange contract.
pub const GRAPH_STRUCTURAL_RERANK_ROUTE: &str = "/graph/structural/rerank";
/// Stable route for the constraint-filter graph-search exchange contract.
pub const GRAPH_STRUCTURAL_FILTER_ROUTE: &str = "/graph/structural/filter";

/// Canonical graph-structural request `query_id` column.
pub const GRAPH_STRUCTURAL_QUERY_ID_COLUMN: &str = "query_id";
/// Canonical graph-structural request or response `candidate_id` column.
pub const GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN: &str = "candidate_id";
/// Canonical graph-structural request `retrieval_layer` column.
pub const GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN: &str = "retrieval_layer";
/// Canonical graph-structural request `query_max_layers` column.
pub const GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN: &str = "query_max_layers";
/// Canonical structural-rerank request `semantic_score` column.
pub const GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN: &str = "semantic_score";
/// Canonical structural-rerank request `dependency_score` column.
pub const GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN: &str = "dependency_score";
/// Canonical structural-rerank request `keyword_score` column.
pub const GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN: &str = "keyword_score";
/// Canonical structural-rerank request `tag_score` column.
pub const GRAPH_STRUCTURAL_TAG_SCORE_COLUMN: &str = "tag_score";
/// Canonical graph-structural request `constraint_kind` column.
pub const GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN: &str = "constraint_kind";
/// Canonical graph-structural request `required_boundary_size` column.
pub const GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN: &str = "required_boundary_size";
/// Canonical graph-structural request `anchor_planes` column.
pub const GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN: &str = "anchor_planes";
/// Canonical graph-structural request `anchor_values` column.
pub const GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN: &str = "anchor_values";
/// Canonical graph-structural request `edge_constraint_kinds` column.
pub const GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN: &str = "edge_constraint_kinds";
/// Canonical graph-structural request `candidate_node_ids` column.
pub const GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN: &str = "candidate_node_ids";
/// Canonical graph-structural request `candidate_edge_sources` column.
pub const GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN: &str = "candidate_edge_sources";
/// Canonical graph-structural request `candidate_edge_destinations` column.
pub const GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN: &str = "candidate_edge_destinations";
/// Canonical graph-structural request `candidate_edge_kinds` column.
pub const GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN: &str = "candidate_edge_kinds";
/// Canonical structural-rerank response `feasible` column.
pub const GRAPH_STRUCTURAL_FEASIBLE_COLUMN: &str = "feasible";
/// Canonical constraint-filter response `accepted` column.
pub const GRAPH_STRUCTURAL_ACCEPTED_COLUMN: &str = "accepted";
/// Canonical graph-structural response `structural_score` column.
pub const GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN: &str = "structural_score";
/// Canonical structural-rerank response `final_score` column.
pub const GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN: &str = "final_score";
/// Canonical graph-structural response `pin_assignment` column.
pub const GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN: &str = "pin_assignment";
/// Canonical structural-rerank response `explanation` column.
pub const GRAPH_STRUCTURAL_EXPLANATION_COLUMN: &str = "explanation";
/// Canonical constraint-filter response `rejection_reason` column.
pub const GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN: &str = "rejection_reason";

/// Canonical structural-rerank request column order.
pub const GRAPH_STRUCTURAL_RERANK_REQUEST_COLUMNS: [&str; 15] = [
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
];

/// Canonical structural-rerank response column order.
pub const GRAPH_STRUCTURAL_RERANK_RESPONSE_COLUMNS: [&str; 6] = [
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
    GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
];

/// Canonical constraint-filter request column order.
pub const GRAPH_STRUCTURAL_FILTER_REQUEST_COLUMNS: [&str; 13] = [
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
];

/// Canonical constraint-filter response column order.
pub const GRAPH_STRUCTURAL_FILTER_RESPONSE_COLUMNS: [&str; 5] = [
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
    GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
];

pub(super) const GRAPH_STRUCTURAL_RERANK_REQUEST_UTF8_COLUMNS: &[&str] = &[
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
];
pub(super) const GRAPH_STRUCTURAL_RERANK_REQUEST_INT32_COLUMNS: &[&str] = &[
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
];
pub(super) const GRAPH_STRUCTURAL_RERANK_REQUEST_FLOAT64_COLUMNS: &[&str] = &[
    GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
];
pub(super) const GRAPH_STRUCTURAL_RERANK_REQUEST_LIST_UTF8_COLUMNS: &[&str] = &[
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
];
pub(super) const GRAPH_STRUCTURAL_FILTER_REQUEST_UTF8_COLUMNS: &[&str] = &[
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN,
];
pub(super) const GRAPH_STRUCTURAL_FILTER_REQUEST_INT32_COLUMNS: &[&str] = &[
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN,
];
pub(super) const GRAPH_STRUCTURAL_FILTER_REQUEST_LIST_UTF8_COLUMNS: &[&str] =
    GRAPH_STRUCTURAL_RERANK_REQUEST_LIST_UTF8_COLUMNS;
