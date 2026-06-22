use crate::{
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    julia_plugin_test_support::common::ResultTestExt,
};

use super::{
    GraphStructuralFilterRequestRow, GraphStructuralFilterScoreRow,
    GraphStructuralRerankRequestRow, GraphStructuralRerankScoreRow,
    build_graph_structural_filter_request_batch, build_graph_structural_rerank_request_batch,
    decode_graph_structural_filter_score_rows, decode_graph_structural_rerank_score_rows,
    filter_response_batch, rerank_response_batch,
};

#[test]
fn build_graph_structural_rerank_request_batch_uses_contract_columns() {
    let batch = build_graph_structural_rerank_request_batch(&[GraphStructuralRerankRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 0,
        query_max_layers: 2,
        semantic_score: 0.7,
        dependency_score: 0.6,
        keyword_score: 0.4,
        tag_score: 0.3,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("rerank request batch");

    assert_eq!(
        batch.schema().field(0).name(),
        GRAPH_STRUCTURAL_QUERY_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(1).name(),
        GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(4).name(),
        GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN
    );
    assert_eq!(
        batch.schema().field(14).name(),
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN
    );
}

#[test]
fn build_graph_structural_filter_request_batch_rejects_misaligned_anchors() {
    let error = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 1,
        query_max_layers: 3,
        constraint_kind: "boundary-match".into(),
        required_boundary_size: 2,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string(), "tag:core".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .err_or_panic("misaligned anchors must fail");

    assert!(
        error
            .to_string()
            .contains("anchor columns must stay aligned"),
        "unexpected error: {error}"
    );
}

#[test]
fn decode_graph_structural_rerank_score_rows_materializes_values() {
    let rows = decode_graph_structural_rerank_score_rows(&[rerank_response_batch()])
        .or_panic("rerank decode");

    assert_eq!(
        rows.get("candidate-a"),
        Some(&GraphStructuralRerankScoreRow {
            candidate_id: "candidate-a".to_string(),
            feasible: true,
            structural_score: 0.91,
            final_score: 0.87,
            pin_assignment: vec!["pin:entry".to_string(), "pin:exit".to_string()],
            explanation: "accepted".to_string(),
        })
    );
}

#[test]
fn decode_graph_structural_filter_score_rows_materializes_values() {
    let rows = decode_graph_structural_filter_score_rows(&[filter_response_batch()])
        .or_panic("filter decode");

    assert_eq!(
        rows.get("candidate-a"),
        Some(&GraphStructuralFilterScoreRow {
            candidate_id: "candidate-a".to_string(),
            accepted: false,
            structural_score: 0.52,
            pin_assignment: vec!["pin:entry".to_string()],
            rejection_reason: "missing boundary".to_string(),
        })
    );
}
