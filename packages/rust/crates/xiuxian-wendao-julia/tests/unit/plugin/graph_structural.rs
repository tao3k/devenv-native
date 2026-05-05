use arrow::array::{
    ArrayRef, BooleanArray, Float64Array, Int32Array, ListArray, StringArray,
    builder::{ListBuilder, StringBuilder},
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use super::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FILTER_RESPONSE_COLUMNS,
    GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN, GRAPH_STRUCTURAL_RERANK_REQUEST_COLUMNS,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    GraphStructuralRouteKind, JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION, graph_structural_route_kind,
    is_graph_structural_route, validate_graph_structural_filter_request_batch,
    validate_graph_structural_filter_response_batch,
    validate_graph_structural_rerank_request_batch,
    validate_graph_structural_rerank_response_batch,
};
use crate::julia_plugin_test_support::common::ResultTestExt;

struct StructuralRerankRequestBatchInputs<'a> {
    query_ids: Vec<&'a str>,
    candidate_ids: Vec<&'a str>,
    retrieval_layers: Vec<i32>,
    query_max_layers: Vec<i32>,
    semantic_scores: Vec<f64>,
    dependency_scores: Vec<f64>,
    keyword_scores: Vec<f64>,
    tag_scores: Vec<f64>,
    anchor_planes: Vec<Vec<&'a str>>,
    anchor_values: Vec<Vec<&'a str>>,
    edge_constraint_kinds: Vec<Vec<&'a str>>,
    candidate_node_ids: Vec<Vec<&'a str>>,
    candidate_edge_sources: Vec<Vec<&'a str>>,
    candidate_edge_destinations: Vec<Vec<&'a str>>,
    candidate_edge_kinds: Vec<Vec<&'a str>>,
}

struct StructuralFilterRequestBatchInputs<'a> {
    query_ids: Vec<&'a str>,
    candidate_ids: Vec<&'a str>,
    retrieval_layers: Vec<i32>,
    query_max_layers: Vec<i32>,
    constraint_kinds: Vec<&'a str>,
    required_boundary_sizes: Vec<i32>,
    anchor_planes: Vec<Vec<&'a str>>,
    anchor_values: Vec<Vec<&'a str>>,
    edge_constraint_kinds: Vec<Vec<&'a str>>,
    candidate_node_ids: Vec<Vec<&'a str>>,
    candidate_edge_sources: Vec<Vec<&'a str>>,
    candidate_edge_destinations: Vec<Vec<&'a str>>,
    candidate_edge_kinds: Vec<Vec<&'a str>>,
}

#[test]
fn graph_structural_route_staging_resolves_canonical_paths() {
    assert_eq!(
        graph_structural_route_kind("graph/structural/rerank"),
        Ok(GraphStructuralRouteKind::StructuralRerank)
    );
    assert_eq!(
        graph_structural_route_kind("/graph/structural/filter"),
        Ok(GraphStructuralRouteKind::ConstraintFilter)
    );
    assert!(is_graph_structural_route("/graph/structural/rerank"));
    assert!(!is_graph_structural_route("/graph/neighbors"));
    assert_eq!(
        GraphStructuralRouteKind::StructuralRerank.request_columns(),
        &GRAPH_STRUCTURAL_RERANK_REQUEST_COLUMNS
    );
    assert_eq!(
        GraphStructuralRouteKind::ConstraintFilter.response_columns(),
        &GRAPH_STRUCTURAL_FILTER_RESPONSE_COLUMNS
    );
    assert_eq!(
        GraphStructuralRouteKind::StructuralRerank.schema_version(),
        JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION
    );
}

#[test]
fn structural_rerank_request_batch_validation_accepts_staged_shape() {
    let batch = structural_rerank_request_batch(StructuralRerankRequestBatchInputs {
        query_ids: vec!["query-1"],
        candidate_ids: vec!["candidate-1"],
        retrieval_layers: vec![1],
        query_max_layers: vec![3],
        semantic_scores: vec![0.9],
        dependency_scores: vec![0.4],
        keyword_scores: vec![0.2],
        tag_scores: vec![0.1],
        anchor_planes: vec![vec!["semantic"]],
        anchor_values: vec![vec!["graph retrieval"]],
        edge_constraint_kinds: vec![vec!["depends_on"]],
        candidate_node_ids: vec![vec!["node-a", "node-b"]],
        candidate_edge_sources: vec![vec!["node-a"]],
        candidate_edge_destinations: vec![vec!["node-b"]],
        candidate_edge_kinds: vec![vec!["depends_on"]],
    });
    assert!(validate_graph_structural_rerank_request_batch(&batch).is_ok());
}

#[test]
fn structural_rerank_request_batch_validation_rejects_misaligned_anchor_lists() {
    let batch = structural_rerank_request_batch(StructuralRerankRequestBatchInputs {
        query_ids: vec!["query-1"],
        candidate_ids: vec!["candidate-1"],
        retrieval_layers: vec![1],
        query_max_layers: vec![3],
        semantic_scores: vec![0.9],
        dependency_scores: vec![0.4],
        keyword_scores: vec![0.2],
        tag_scores: vec![0.1],
        anchor_planes: vec![vec!["semantic", "keyword"]],
        anchor_values: vec![vec!["graph retrieval"]],
        edge_constraint_kinds: vec![vec!["depends_on"]],
        candidate_node_ids: vec![vec!["node-a", "node-b"]],
        candidate_edge_sources: vec![vec!["node-a"]],
        candidate_edge_destinations: vec![vec!["node-b"]],
        candidate_edge_kinds: vec![vec!["depends_on"]],
    });
    assert_eq!(
            validate_graph_structural_rerank_request_batch(&batch),
            Err(
                "graph structural rerank request anchor columns must stay aligned; row 0 has 2 planes but 1 values"
                    .to_string()
            )
        );
}

#[test]
fn structural_rerank_response_batch_validation_rejects_non_finite_final_score() {
    let batch = structural_rerank_response_batch(
        vec!["candidate-1"],
        vec![true],
        vec![0.8],
        vec![f64::INFINITY],
        vec![vec!["node-a"]],
        vec!["matched"],
    );
    assert_eq!(
        validate_graph_structural_rerank_response_batch(&batch),
        Err(
            "graph structural column `final_score` must contain finite values; row 0 is inf"
                .to_string()
        )
    );
}

#[test]
fn structural_filter_request_batch_validation_accepts_staged_shape() {
    let batch = structural_filter_request_batch(StructuralFilterRequestBatchInputs {
        query_ids: vec!["query-1"],
        candidate_ids: vec!["candidate-1"],
        retrieval_layers: vec![0],
        query_max_layers: vec![2],
        constraint_kinds: vec!["pin_assignment"],
        required_boundary_sizes: vec![2],
        anchor_planes: vec![vec!["semantic"]],
        anchor_values: vec![vec!["graph retrieval"]],
        edge_constraint_kinds: vec![vec!["depends_on"]],
        candidate_node_ids: vec![vec!["node-a", "node-b"]],
        candidate_edge_sources: vec![vec!["node-a"]],
        candidate_edge_destinations: vec![vec!["node-b"]],
        candidate_edge_kinds: vec![vec!["depends_on"]],
    });
    assert!(validate_graph_structural_filter_request_batch(&batch).is_ok());
}

#[test]
fn structural_filter_response_batch_validation_rejects_missing_rejection_reason() {
    let batch = structural_filter_response_batch(
        vec!["candidate-1"],
        vec![false],
        vec![0.4],
        vec![vec!["node-a"]],
        vec![""],
    );
    assert_eq!(
            validate_graph_structural_filter_response_batch(&batch),
            Err(
                "graph structural filter response column `rejection_reason` must be non-blank for rejected candidate `candidate-1` at row 0"
                    .to_string()
            )
        );
}

#[test]
fn structural_filter_response_batch_validation_rejects_duplicate_candidate_id() {
    let batch = structural_filter_response_batch(
        vec!["candidate-1", "candidate-1"],
        vec![true, false],
        vec![0.8, 0.4],
        vec![vec!["node-a"], vec!["node-b"]],
        vec!["", "gap"],
    );
    assert_eq!(
            validate_graph_structural_filter_response_batch(&batch),
            Err(
                "graph structural column `candidate_id` must be unique across one batch; row 1 duplicates `candidate-1`"
                    .to_string()
            )
        );
}

fn structural_rerank_request_batch(inputs: StructuralRerankRequestBatchInputs<'_>) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_QUERY_ID_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            int32_field(GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN),
            int32_field(GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN),
            float64_field(GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_TAG_SCORE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(inputs.query_ids)) as ArrayRef,
            Arc::new(StringArray::from(inputs.candidate_ids)) as ArrayRef,
            Arc::new(Int32Array::from(inputs.retrieval_layers)) as ArrayRef,
            Arc::new(Int32Array::from(inputs.query_max_layers)) as ArrayRef,
            Arc::new(Float64Array::from(inputs.semantic_scores)) as ArrayRef,
            Arc::new(Float64Array::from(inputs.dependency_scores)) as ArrayRef,
            Arc::new(Float64Array::from(inputs.keyword_scores)) as ArrayRef,
            Arc::new(Float64Array::from(inputs.tag_scores)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.anchor_planes)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.anchor_values)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.edge_constraint_kinds)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_node_ids)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_edge_sources)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_edge_destinations)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_edge_kinds)) as ArrayRef,
        ],
    )
    .or_panic("structural rerank request batch should build")
}

fn structural_rerank_response_batch(
    candidate_ids: Vec<&str>,
    feasible: Vec<bool>,
    structural_scores: Vec<f64>,
    final_scores: Vec<f64>,
    pin_assignments: Vec<Vec<&str>>,
    explanations: Vec<&str>,
) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            bool_field(GRAPH_STRUCTURAL_FEASIBLE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_EXPLANATION_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(candidate_ids)) as ArrayRef,
            Arc::new(BooleanArray::from(feasible)) as ArrayRef,
            Arc::new(Float64Array::from(structural_scores)) as ArrayRef,
            Arc::new(Float64Array::from(final_scores)) as ArrayRef,
            Arc::new(list_utf8_array(pin_assignments)) as ArrayRef,
            Arc::new(StringArray::from(explanations)) as ArrayRef,
        ],
    )
    .or_panic("structural rerank response batch should build")
}

fn structural_filter_request_batch(inputs: StructuralFilterRequestBatchInputs<'_>) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_QUERY_ID_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            int32_field(GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN),
            int32_field(GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN),
            int32_field(GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(inputs.query_ids)) as ArrayRef,
            Arc::new(StringArray::from(inputs.candidate_ids)) as ArrayRef,
            Arc::new(Int32Array::from(inputs.retrieval_layers)) as ArrayRef,
            Arc::new(Int32Array::from(inputs.query_max_layers)) as ArrayRef,
            Arc::new(StringArray::from(inputs.constraint_kinds)) as ArrayRef,
            Arc::new(Int32Array::from(inputs.required_boundary_sizes)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.anchor_planes)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.anchor_values)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.edge_constraint_kinds)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_node_ids)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_edge_sources)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_edge_destinations)) as ArrayRef,
            Arc::new(list_utf8_array(inputs.candidate_edge_kinds)) as ArrayRef,
        ],
    )
    .or_panic("structural filter request batch should build")
}

fn structural_filter_response_batch(
    candidate_ids: Vec<&str>,
    accepted: Vec<bool>,
    structural_scores: Vec<f64>,
    pin_assignments: Vec<Vec<&str>>,
    rejection_reasons: Vec<&str>,
) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            bool_field(GRAPH_STRUCTURAL_ACCEPTED_COLUMN),
            float64_field(GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(candidate_ids)) as ArrayRef,
            Arc::new(BooleanArray::from(accepted)) as ArrayRef,
            Arc::new(Float64Array::from(structural_scores)) as ArrayRef,
            Arc::new(list_utf8_array(pin_assignments)) as ArrayRef,
            Arc::new(StringArray::from(rejection_reasons)) as ArrayRef,
        ],
    )
    .or_panic("structural filter response batch should build")
}

fn utf8_field(name: &str) -> Field {
    Field::new(name, DataType::Utf8, false)
}

fn bool_field(name: &str) -> Field {
    Field::new(name, DataType::Boolean, false)
}

fn int32_field(name: &str) -> Field {
    Field::new(name, DataType::Int32, false)
}

fn float64_field(name: &str) -> Field {
    Field::new(name, DataType::Float64, false)
}

fn list_utf8_field(name: &str) -> Field {
    Field::new(
        name,
        DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
        false,
    )
}

fn list_utf8_array(rows: Vec<Vec<&str>>) -> ListArray {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for row in rows {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
