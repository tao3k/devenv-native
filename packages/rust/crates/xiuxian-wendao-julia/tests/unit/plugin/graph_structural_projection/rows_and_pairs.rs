use super::{
    GraphStructuralCandidateSubgraph, GraphStructuralFilterConstraint,
    GraphStructuralPairCandidateInputs, GraphStructuralQueryAnchor, GraphStructuralQueryContext,
    GraphStructuralRerankSignals, build_graph_structural_filter_request_batch,
    build_graph_structural_filter_request_row, build_graph_structural_pair_candidate_inputs,
    build_graph_structural_pair_candidate_subgraph, build_graph_structural_pair_filter_request_row,
    build_graph_structural_pair_rerank_request_row, build_graph_structural_rerank_request_batch,
    build_graph_structural_rerank_request_row, build_graph_structural_scored_pair_candidate_inputs,
};
use crate::julia_plugin_test_support::common::ResultTestExt;

#[test]
fn build_graph_structural_rerank_request_row_projects_semantic_dtos() {
    let query = GraphStructuralQueryContext::new(
        "query-1",
        1,
        3,
        vec![
            GraphStructuralQueryAnchor::new("semantic", "symbol:entry").or_panic("semantic anchor"),
            GraphStructuralQueryAnchor::new("tag", "core").or_panic("tag anchor"),
        ],
        vec!["depends_on".to_string()],
    )
    .or_panic("query context");
    let candidate = GraphStructuralCandidateSubgraph::new(
        "pair:node-1:node-2",
        vec!["node-1".to_string(), "node-2".to_string()],
        vec!["node-1".to_string()],
        vec!["node-2".to_string()],
        vec!["related".to_string()],
    )
    .or_panic("candidate");
    let signals = GraphStructuralRerankSignals::new(0.7, 0.4, 0.2, 0.3).or_panic("rerank signals");

    let row = build_graph_structural_rerank_request_row(&query, &candidate, &signals);
    let batch = build_graph_structural_rerank_request_batch(std::slice::from_ref(&row))
        .or_panic("rerank batch should validate");

    assert_eq!(row.query_id, "query-1");
    assert_eq!(row.candidate_id, "pair:node-1:node-2");
    assert_eq!(row.anchor_planes, vec!["semantic", "tag"]);
    assert_eq!(row.anchor_values, vec!["symbol:entry", "core"]);
    assert_eq!(row.edge_constraint_kinds, vec!["depends_on"]);
    assert_eq!(row.candidate_node_ids, vec!["node-1", "node-2"]);
    assert_eq!(row.candidate_edge_sources, vec!["node-1"]);
    assert_eq!(row.candidate_edge_destinations, vec!["node-2"]);
    assert_eq!(batch.num_rows(), 1);
}

#[test]
fn build_graph_structural_filter_request_row_allows_empty_edge_lists() {
    let query = GraphStructuralQueryContext::new(
        "query-2",
        0,
        2,
        vec![GraphStructuralQueryAnchor::new("keyword", "solver").or_panic("keyword anchor")],
        Vec::new(),
    )
    .or_panic("query context");
    let candidate = GraphStructuralCandidateSubgraph::new(
        "candidate-a",
        vec!["node-a".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .or_panic("candidate");
    let constraint =
        GraphStructuralFilterConstraint::new("boundary-match", 1).or_panic("constraint");

    let row = build_graph_structural_filter_request_row(&query, &candidate, &constraint);
    let batch = build_graph_structural_filter_request_batch(std::slice::from_ref(&row))
        .or_panic("filter batch should validate");

    assert_eq!(row.edge_constraint_kinds, Vec::<String>::new());
    assert_eq!(row.candidate_edge_sources, Vec::<String>::new());
    assert_eq!(row.candidate_edge_destinations, Vec::<String>::new());
    assert_eq!(row.candidate_edge_kinds, Vec::<String>::new());
    assert_eq!(row.constraint_kind, "boundary-match");
    assert_eq!(row.required_boundary_size, 1);
    assert_eq!(batch.num_rows(), 1);
}

#[test]
fn build_graph_structural_pair_candidate_subgraph_normalizes_stable_id() {
    let candidate = build_graph_structural_pair_candidate_subgraph(
        "node-z",
        "node-a",
        vec!["related".to_string()],
    )
    .or_panic("pair candidate should normalize");

    assert_eq!(candidate.candidate_id(), "pair:node-a:node-z");
    assert_eq!(
        candidate.node_ids(),
        &["node-z".to_string(), "node-a".to_string()]
    );
    assert_eq!(candidate.edge_kinds(), &["related".to_string()]);
}

#[test]
fn build_graph_structural_pair_rerank_request_row_projects_pair_inputs() {
    let query = GraphStructuralQueryContext::new(
        "query-3",
        0,
        2,
        vec![GraphStructuralQueryAnchor::new("keyword", "alpha").or_panic("keyword anchor")],
        Vec::new(),
    )
    .or_panic("query context");
    let signals = GraphStructuralRerankSignals::new(0.8, 0.1, 1.0, 0.6).or_panic("rerank signals");

    let row = build_graph_structural_pair_rerank_request_row(
        &query,
        "doc-b",
        "doc-a",
        vec!["semantic_similar".to_string()],
        &signals,
    )
    .or_panic("pair row should project");
    let batch = build_graph_structural_rerank_request_batch(std::slice::from_ref(&row))
        .or_panic("pair rerank batch should validate");

    assert_eq!(row.candidate_id, "pair:doc-a:doc-b");
    assert_eq!(row.candidate_node_ids, vec!["doc-b", "doc-a"]);
    assert_eq!(row.candidate_edge_kinds, vec!["semantic_similar"]);
    assert_eq!(batch.num_rows(), 1);
}

#[test]
fn build_graph_structural_scored_pair_candidate_inputs_rejects_negative_score() {
    let error = build_graph_structural_scored_pair_candidate_inputs(
        "node-1",
        "node-2",
        vec!["depends_on".to_string()],
        -0.1,
    )
    .err_or_panic("negative pair score should fail");

    assert!(
        error
            .to_string()
            .contains("pair semantic score must be non-negative"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_graph_structural_pair_filter_request_row_rejects_duplicate_endpoints() {
    let query = GraphStructuralQueryContext::new(
        "query-4",
        0,
        1,
        vec![GraphStructuralQueryAnchor::new("tag", "core").or_panic("tag anchor")],
        Vec::new(),
    )
    .or_panic("query context");
    let constraint =
        GraphStructuralFilterConstraint::new("boundary-match", 1).or_panic("constraint");

    let error = build_graph_structural_pair_filter_request_row(
        &query,
        "node-a",
        "node-a",
        Vec::new(),
        &constraint,
    )
    .err_or_panic("pair filter row should reject duplicate endpoints");
    assert!(
        error
            .to_string()
            .contains("pair endpoints must not resolve to the same id"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_graph_structural_pair_candidate_inputs_composes() {
    let pair_inputs = build_graph_structural_pair_candidate_inputs(
        "node-left",
        "node-right",
        vec!["semantic_similar".to_string()],
    );

    assert_eq!(
        pair_inputs,
        GraphStructuralPairCandidateInputs::new(
            "node-left",
            "node-right",
            vec!["semantic_similar".to_string()],
        )
    );
}
