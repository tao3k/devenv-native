//! Graph-structural request and response row exchange helpers.

use std::collections::BTreeMap;
use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Float64Array, Int32Array, ListArray, ListBuilder, StringArray,
    StringBuilder,
};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};

use crate::JuliaContractKind;

use super::graph_structural::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_EXPLANATION_COLUMN, GRAPH_STRUCTURAL_FEASIBLE_COLUMN,
    GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
    GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
    GraphStructuralRouteKind, graph_structural_filter_request_schema,
    graph_structural_rerank_request_schema, validate_graph_structural_filter_request_batch,
    validate_graph_structural_filter_response_batch,
    validate_graph_structural_rerank_request_batch,
    validate_graph_structural_rerank_response_batch,
};
use super::graph_structural_projection::{
    GraphStructuralFilterConstraint, GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralKeywordOverlapCandidateInputs, GraphStructuralKeywordOverlapQueryInputs,
    GraphStructuralKeywordOverlapRawCandidateInputs,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    build_graph_structural_generic_topology_filter_request_batch,
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_batch,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
};
use super::graph_structural_transport::process_graph_structural_flight_batches_for_repository;

/// One typed request row for the structural-rerank Julia graph-structural contract.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRerankRequestRow {
    /// Stable query identity used across the staged rerank request.
    pub query_id: String,
    /// Candidate subgraph identity within the current retrieval batch.
    pub candidate_id: String,
    /// Retrieval depth where the candidate was surfaced.
    pub retrieval_layer: i32,
    /// Maximum layered expansion depth allowed for the query.
    pub query_max_layers: i32,
    /// Semantic-plane score contributed by the host retrieval stage.
    pub semantic_score: f64,
    /// Dependency-plane score contributed by the host retrieval stage.
    pub dependency_score: f64,
    /// Keyword-plane score contributed by the host retrieval stage.
    pub keyword_score: f64,
    /// Tag-plane score contributed by the host retrieval stage.
    pub tag_score: f64,
    /// Ordered anchor planes aligned with `anchor_values`.
    pub anchor_planes: Vec<String>,
    /// Ordered anchor values aligned with `anchor_planes`.
    pub anchor_values: Vec<String>,
    /// Edge-relation constraints that the structural stage should honor.
    pub edge_constraint_kinds: Vec<String>,
    /// Node identifiers contained in the candidate subgraph.
    pub candidate_node_ids: Vec<String>,
    /// Edge source identifiers aligned with `candidate_edge_kinds`.
    pub candidate_edge_sources: Vec<String>,
    /// Edge destination identifiers aligned with `candidate_edge_kinds`.
    pub candidate_edge_destinations: Vec<String>,
    /// Edge kinds contained in the candidate subgraph.
    pub candidate_edge_kinds: Vec<String>,
}

/// One typed request row for the constraint-filter Julia graph-structural contract.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralFilterRequestRow {
    /// Stable query identity used across the staged filter request.
    pub query_id: String,
    /// Candidate subgraph identity within the current retrieval batch.
    pub candidate_id: String,
    /// Retrieval depth where the candidate was surfaced.
    pub retrieval_layer: i32,
    /// Maximum layered expansion depth allowed for the query.
    pub query_max_layers: i32,
    /// Constraint family that should be enforced against the candidate.
    pub constraint_kind: JuliaContractKind,
    /// Boundary size required by the staged constraint contract.
    pub required_boundary_size: i32,
    /// Ordered anchor planes aligned with `anchor_values`.
    pub anchor_planes: Vec<String>,
    /// Ordered anchor values aligned with `anchor_planes`.
    pub anchor_values: Vec<String>,
    /// Edge-relation constraints that the structural stage should honor.
    pub edge_constraint_kinds: Vec<String>,
    /// Node identifiers contained in the candidate subgraph.
    pub candidate_node_ids: Vec<String>,
    /// Edge source identifiers aligned with `candidate_edge_kinds`.
    pub candidate_edge_sources: Vec<String>,
    /// Edge destination identifiers aligned with `candidate_edge_kinds`.
    pub candidate_edge_destinations: Vec<String>,
    /// Edge kinds contained in the candidate subgraph.
    pub candidate_edge_kinds: Vec<String>,
}

/// One typed response row for structural-rerank Julia graph-structural results.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRerankScoreRow {
    /// Candidate subgraph identity echoed from the response batch.
    pub candidate_id: String,
    /// Whether the solver judged the candidate structurally feasible.
    pub feasible: bool,
    /// Pure structural score returned by the Julia stage.
    pub structural_score: f64,
    /// Final fused score returned by the Julia stage.
    pub final_score: f64,
    /// Ordered boundary or pin assignment chosen for the candidate.
    pub pin_assignment: Vec<String>,
    /// Human-readable explanation attached to the rerank decision.
    pub explanation: String,
}

/// One typed response row for constraint-filter Julia graph-structural results.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralFilterScoreRow {
    /// Candidate subgraph identity echoed from the response batch.
    pub candidate_id: String,
    /// Whether the candidate satisfied the staged filter constraints.
    pub accepted: bool,
    /// Pure structural score returned by the Julia stage.
    pub structural_score: f64,
    /// Ordered boundary or pin assignment chosen for the candidate.
    pub pin_assignment: Vec<String>,
    /// Rejection explanation returned when `accepted` is false.
    pub rejection_reason: String,
}

/// Build one structural-rerank request batch from typed host rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the batch cannot be materialized or
/// violates the Julia-owned staged structural-rerank contract.
pub fn build_graph_structural_rerank_request_batch(
    rows: &[GraphStructuralRerankRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        Arc::new(graph_structural_rerank_request_schema()),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.query_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.candidate_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                rows.iter()
                    .map(|row| row.retrieval_layer)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                rows.iter()
                    .map(|row| row.query_max_layers)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter()
                    .map(|row| row.semantic_score)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter()
                    .map(|row| row.dependency_score)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.keyword_score).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.tag_score).collect::<Vec<_>>(),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.anchor_planes.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.anchor_values.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.edge_constraint_kinds.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.candidate_node_ids.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.candidate_edge_sources.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter()
                    .map(|row| row.candidate_edge_destinations.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.candidate_edge_kinds.as_slice()),
            )),
        ],
    )
    .map_err(|error| {
        graph_structural_request_error(
            GraphStructuralRouteKind::StructuralRerank,
            error.to_string(),
        )
    })?;
    validate_graph_structural_rerank_request_batch(&batch).map_err(|error| {
        graph_structural_request_error(GraphStructuralRouteKind::StructuralRerank, error)
    })?;
    Ok(batch)
}

/// Build one constraint-filter request batch from typed host rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the batch cannot be materialized or
/// violates the Julia-owned staged constraint-filter contract.
pub fn build_graph_structural_filter_request_batch(
    rows: &[GraphStructuralFilterRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        Arc::new(graph_structural_filter_request_schema()),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.query_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.candidate_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                rows.iter()
                    .map(|row| row.retrieval_layer)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                rows.iter()
                    .map(|row| row.query_max_layers)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.constraint_kind.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                rows.iter()
                    .map(|row| row.required_boundary_size)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.anchor_planes.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.anchor_values.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.edge_constraint_kinds.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.candidate_node_ids.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.candidate_edge_sources.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter()
                    .map(|row| row.candidate_edge_destinations.as_slice()),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.candidate_edge_kinds.as_slice()),
            )),
        ],
    )
    .map_err(|error| {
        graph_structural_request_error(
            GraphStructuralRouteKind::ConstraintFilter,
            error.to_string(),
        )
    })?;
    validate_graph_structural_filter_request_batch(&batch).map_err(|error| {
        graph_structural_request_error(GraphStructuralRouteKind::ConstraintFilter, error)
    })?;
    Ok(batch)
}

/// Decode structural-rerank response batches into a `candidate_id` keyed map.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the response batches violate the
/// Julia-owned staged structural-rerank response contract.
pub fn decode_graph_structural_rerank_score_rows(
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, RepoIntelligenceError> {
    let mut rows = BTreeMap::new();
    for batch in batches {
        validate_graph_structural_rerank_response_batch(batch).map_err(|error| {
            graph_structural_decode_error(GraphStructuralRouteKind::StructuralRerank, error)
        })?;

        let candidate_id = utf8_column(
            batch,
            GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
            GraphStructuralRouteKind::StructuralRerank,
        )?;
        let feasible = bool_column(
            batch,
            GRAPH_STRUCTURAL_FEASIBLE_COLUMN,
            GraphStructuralRouteKind::StructuralRerank,
        )?;
        let structural_score = float64_column(
            batch,
            GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
            GraphStructuralRouteKind::StructuralRerank,
        )?;
        let final_score = float64_column(
            batch,
            GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN,
            GraphStructuralRouteKind::StructuralRerank,
        )?;
        let pin_assignment = utf8_list_column(
            batch,
            GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
            GraphStructuralRouteKind::StructuralRerank,
        )?;
        let explanation = utf8_column(
            batch,
            GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
            GraphStructuralRouteKind::StructuralRerank,
        )?;

        for (row, assignment) in pin_assignment.iter().enumerate().take(batch.num_rows()) {
            rows.insert(
                candidate_id.value(row).to_string(),
                GraphStructuralRerankScoreRow {
                    candidate_id: candidate_id.value(row).to_string(),
                    feasible: feasible.value(row),
                    structural_score: structural_score.value(row),
                    final_score: final_score.value(row),
                    pin_assignment: assignment.clone(),
                    explanation: explanation.value(row).to_string(),
                },
            );
        }
    }
    Ok(rows)
}

/// Decode constraint-filter response batches into a `candidate_id` keyed map.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the response batches violate the
/// Julia-owned staged constraint-filter response contract.
pub fn decode_graph_structural_filter_score_rows(
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, GraphStructuralFilterScoreRow>, RepoIntelligenceError> {
    let mut rows = BTreeMap::new();
    for batch in batches {
        validate_graph_structural_filter_response_batch(batch).map_err(|error| {
            graph_structural_decode_error(GraphStructuralRouteKind::ConstraintFilter, error)
        })?;

        let candidate_id = utf8_column(
            batch,
            GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
            GraphStructuralRouteKind::ConstraintFilter,
        )?;
        let accepted = bool_column(
            batch,
            GRAPH_STRUCTURAL_ACCEPTED_COLUMN,
            GraphStructuralRouteKind::ConstraintFilter,
        )?;
        let structural_score = float64_column(
            batch,
            GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
            GraphStructuralRouteKind::ConstraintFilter,
        )?;
        let pin_assignment = utf8_list_column(
            batch,
            GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
            GraphStructuralRouteKind::ConstraintFilter,
        )?;
        let rejection_reason = utf8_column(
            batch,
            GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
            GraphStructuralRouteKind::ConstraintFilter,
        )?;

        for (row, assignment) in pin_assignment.iter().enumerate().take(batch.num_rows()) {
            rows.insert(
                candidate_id.value(row).to_string(),
                GraphStructuralFilterScoreRow {
                    candidate_id: candidate_id.value(row).to_string(),
                    accepted: accepted.value(row),
                    structural_score: structural_score.value(row),
                    pin_assignment: assignment.clone(),
                    rejection_reason: rejection_reason.value(row).to_string(),
                },
            );
        }
    }
    Ok(rows)
}

/// Execute the repository-configured Julia graph-structural rerank transport
/// and decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the remote roundtrip fails or the
/// decoded response violates the staged structural-rerank contract.
pub async fn fetch_graph_structural_rerank_rows_for_repository(
    repository: &RegisteredRepository,
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, RepoIntelligenceError> {
    let response_batches = process_graph_structural_flight_batches_for_repository(
        repository,
        GraphStructuralRouteKind::StructuralRerank,
        batches,
    )
    .await?;
    decode_graph_structural_rerank_score_rows(response_batches.as_slice())
}

/// Build one query-plus-generic-topology structural-rerank request batch,
/// execute the repository-configured Julia graph-structural rerank transport,
/// and decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query-plus-generic-topology
/// projection fails staged Julia-owned normalization, the remote roundtrip
/// fails, or the decoded response violates the staged structural-rerank
/// contract.
pub async fn fetch_graph_structural_generic_topology_rerank_rows_for_repository(
    repository: &RegisteredRepository,
    query: &super::graph_structural_projection::GraphStructuralQueryContext,
    candidates: &[GraphStructuralGenericTopologyCandidateInputs],
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, RepoIntelligenceError> {
    let request_batch =
        build_graph_structural_generic_topology_rerank_request_batch(query, candidates)?;
    let request_batches = vec![request_batch];
    fetch_graph_structural_rerank_rows_for_repository(repository, request_batches.as_slice()).await
}

/// Build one query-plus-raw-connected-pair-collection structural-rerank
/// request batch, execute the repository-configured Julia graph-structural
/// rerank transport, and decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query-plus-collection projection
/// fails staged Julia-owned normalization, the remote roundtrip fails, or the
/// decoded response violates the staged structural-rerank contract.
pub async fn fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections(
    repository: &RegisteredRepository,
    query: &super::graph_structural_projection::GraphStructuralQueryContext,
    candidates: &[GraphStructuralRawConnectedPairCollectionCandidateInputs],
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, RepoIntelligenceError> {
    let request_batch =
        build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections(
            query, candidates,
        )?;
    let request_batches = vec![request_batch];
    fetch_graph_structural_rerank_rows_for_repository(repository, request_batches.as_slice()).await
}

/// Build one query-plus-generic-topology structural-filter request batch,
/// execute the repository-configured Julia graph-structural filter transport,
/// and decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query-plus-generic-topology
/// projection fails staged Julia-owned normalization, the remote roundtrip
/// fails, or the decoded response violates the staged constraint-filter
/// contract.
pub async fn fetch_graph_structural_generic_topology_filter_rows_for_repository(
    repository: &RegisteredRepository,
    query: &super::graph_structural_projection::GraphStructuralQueryContext,
    constraint: &GraphStructuralFilterConstraint,
    candidates: &[GraphStructuralGenericTopologyCandidateInputs],
) -> Result<BTreeMap<String, GraphStructuralFilterScoreRow>, RepoIntelligenceError> {
    let request_batch = build_graph_structural_generic_topology_filter_request_batch(
        query, constraint, candidates,
    )?;
    let request_batches = vec![request_batch];
    fetch_graph_structural_filter_rows_for_repository(repository, request_batches.as_slice()).await
}

/// Build one query-plus-raw-connected-pair-collection structural-filter
/// request batch, execute the repository-configured Julia graph-structural
/// filter transport, and decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query-plus-collection projection
/// fails staged Julia-owned normalization, the remote roundtrip fails, or the
/// decoded response violates the staged constraint-filter contract.
pub async fn fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections(
    repository: &RegisteredRepository,
    query: &super::graph_structural_projection::GraphStructuralQueryContext,
    constraint: &GraphStructuralFilterConstraint,
    candidates: &[GraphStructuralRawConnectedPairCollectionCandidateInputs],
) -> Result<BTreeMap<String, GraphStructuralFilterScoreRow>, RepoIntelligenceError> {
    let request_batch =
        build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections(
            query, constraint, candidates,
        )?;
    let request_batches = vec![request_batch];
    fetch_graph_structural_filter_rows_for_repository(repository, request_batches.as_slice()).await
}

/// Build one query-plus-candidate structural-rerank request batch, execute the
/// repository-configured Julia graph-structural rerank transport, and decode
/// the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query-plus-candidate projection
/// fails staged Julia-owned normalization, the remote roundtrip fails, or the
/// decoded response violates the staged structural-rerank contract.
pub async fn fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository(
    repository: &RegisteredRepository,
    query: &GraphStructuralKeywordOverlapQueryInputs,
    candidates: &[GraphStructuralKeywordOverlapCandidateInputs],
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, RepoIntelligenceError> {
    let request_batch =
        build_graph_structural_keyword_overlap_pair_rerank_request_batch(query, candidates)?;
    let request_batches = vec![request_batch];
    fetch_graph_structural_rerank_rows_for_repository(repository, request_batches.as_slice()).await
}

/// Build one query-plus-raw-candidate structural-rerank request batch, execute
/// the repository-configured Julia graph-structural rerank transport, and
/// decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query-plus-raw-candidate
/// projection fails staged Julia-owned normalization, the remote roundtrip
/// fails, or the decoded response violates the staged structural-rerank
/// contract.
pub async fn fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
    repository: &RegisteredRepository,
    query: &GraphStructuralKeywordOverlapQueryInputs,
    candidates: &[GraphStructuralKeywordOverlapRawCandidateInputs],
) -> Result<BTreeMap<String, GraphStructuralRerankScoreRow>, RepoIntelligenceError> {
    let request_batch =
        build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(
            query, candidates,
        )?;
    let request_batches = vec![request_batch];
    fetch_graph_structural_rerank_rows_for_repository(repository, request_batches.as_slice()).await
}

/// Execute the repository-configured Julia graph-structural filter transport
/// and decode the staged response rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the remote roundtrip fails or the
/// decoded response violates the staged constraint-filter contract.
pub async fn fetch_graph_structural_filter_rows_for_repository(
    repository: &RegisteredRepository,
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, GraphStructuralFilterScoreRow>, RepoIntelligenceError> {
    let response_batches = process_graph_structural_flight_batches_for_repository(
        repository,
        GraphStructuralRouteKind::ConstraintFilter,
        batches,
    )
    .await?;
    decode_graph_structural_filter_score_rows(response_batches.as_slice())
}

fn build_utf8_list_array<'a>(rows: impl IntoIterator<Item = &'a [String]>) -> ListArray {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for row in rows {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}

fn utf8_column<'a>(
    batch: &'a RecordBatch,
    column: &str,
    route_kind: GraphStructuralRouteKind,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            graph_structural_decode_error(
                route_kind,
                format!("missing required Utf8 column `{column}`"),
            )
        })
}

fn bool_column<'a>(
    batch: &'a RecordBatch,
    column: &str,
    route_kind: GraphStructuralRouteKind,
) -> Result<&'a BooleanArray, RepoIntelligenceError> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<BooleanArray>())
        .ok_or_else(|| {
            graph_structural_decode_error(
                route_kind,
                format!("missing required Boolean column `{column}`"),
            )
        })
}

fn float64_column<'a>(
    batch: &'a RecordBatch,
    column: &str,
    route_kind: GraphStructuralRouteKind,
) -> Result<&'a Float64Array, RepoIntelligenceError> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| {
            graph_structural_decode_error(
                route_kind,
                format!("missing required Float64 column `{column}`"),
            )
        })
}

fn utf8_list_column(
    batch: &RecordBatch,
    column: &str,
    route_kind: GraphStructuralRouteKind,
) -> Result<Vec<Vec<String>>, RepoIntelligenceError> {
    let array = batch
        .column_by_name(column)
        .and_then(|value| value.as_any().downcast_ref::<ListArray>())
        .ok_or_else(|| {
            graph_structural_decode_error(
                route_kind,
                format!("missing required List<Utf8> column `{column}`"),
            )
        })?;
    let mut rows = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        if array.is_null(row) {
            return Err(graph_structural_decode_error(
                route_kind,
                format!("column `{column}` must be non-null at row {row}"),
            ));
        }
        let values = array.value(row);
        let values = values
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                graph_structural_decode_error(
                    route_kind,
                    format!("column `{column}` must contain Utf8 values"),
                )
            })?;
        let mut row_values = Vec::with_capacity(values.len());
        for value_index in 0..values.len() {
            if values.is_null(value_index) {
                return Err(graph_structural_decode_error(
                    route_kind,
                    format!("column `{column}` contains null Utf8 value at row {row}"),
                ));
            }
            row_values.push(values.value(value_index).to_string());
        }
        rows.push(row_values);
    }
    Ok(rows)
}

fn graph_structural_request_error(
    route_kind: GraphStructuralRouteKind,
    detail: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to build Julia graph-structural {} request batch: {}",
            route_kind.route(),
            detail.into()
        ),
    }
}

fn graph_structural_decode_error(
    route_kind: GraphStructuralRouteKind,
    detail: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to decode Julia graph-structural {} response batch: {}",
            route_kind.route(),
            detail.into()
        ),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/graph_structural_exchange.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/plugin/graph_structural_exchange_generic_topology.rs"]
mod generic_topology_tests;
