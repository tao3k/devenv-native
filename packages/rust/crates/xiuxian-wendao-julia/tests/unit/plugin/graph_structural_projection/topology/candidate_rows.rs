use super::{
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInputs, ResultTestExt, assert_f64_eq,
    build_graph_structural_generic_topology_candidate_inputs,
    build_graph_structural_generic_topology_candidate_metadata_inputs,
    build_graph_structural_generic_topology_candidate_subgraph,
    build_graph_structural_generic_topology_rerank_request_row,
    build_graph_structural_keyword_tag_query_context,
};
#[test]
fn build_graph_structural_generic_topology_candidate_subgraph_projects_explicit_edges() {
    let candidate = build_graph_structural_generic_topology_candidate_subgraph(
        build_graph_structural_generic_topology_candidate_metadata_inputs(
            "candidate-chain",
            vec![
                "node-1".to_string(),
                "node-2".to_string(),
                "node-3".to_string(),
            ],
            vec!["node-1".to_string(), "node-2".to_string()],
            vec!["node-2".to_string(), "node-3".to_string()],
            vec!["depends_on".to_string(), "references".to_string()],
        ),
    )
    .or_panic("generic topology candidate should normalize");

    assert_eq!(candidate.candidate_id(), "candidate-chain");
    assert_eq!(
        candidate.node_ids(),
        &[
            "node-1".to_string(),
            "node-2".to_string(),
            "node-3".to_string()
        ]
    );
    assert_eq!(
        candidate.edge_sources(),
        &["node-1".to_string(), "node-2".to_string()]
    );
    assert_eq!(
        candidate.edge_destinations(),
        &["node-2".to_string(), "node-3".to_string()]
    );
    assert_eq!(
        candidate.edge_kinds(),
        &["depends_on".to_string(), "references".to_string()]
    );
}

#[test]
fn build_graph_structural_generic_topology_rerank_request_row_projects_explicit_topology() {
    let query = build_graph_structural_keyword_tag_query_context(
        "query-generic-row",
        1,
        3,
        vec!["alpha".to_string()],
        vec!["core".to_string()],
        vec!["depends_on".to_string()],
    )
    .or_panic("query context");
    let row = build_graph_structural_generic_topology_rerank_request_row(
        &query,
        build_graph_structural_generic_topology_candidate_inputs(
            build_graph_structural_generic_topology_candidate_metadata_inputs(
                "candidate-chain",
                vec![
                    "node-1".to_string(),
                    "node-2".to_string(),
                    "node-3".to_string(),
                ],
                vec!["node-1".to_string(), "node-2".to_string()],
                vec!["node-2".to_string(), "node-3".to_string()],
                vec!["depends_on".to_string(), "depends_on".to_string()],
            ),
            0.7,
            0.5,
            1.0,
            0.0,
        ),
    )
    .or_panic("generic topology rerank row should normalize");

    assert_eq!(row.candidate_id, "candidate-chain");
    assert_eq!(row.candidate_node_ids.len(), 3);
    assert_eq!(row.candidate_edge_sources, vec!["node-1", "node-2"]);
    assert_eq!(row.candidate_edge_destinations, vec!["node-2", "node-3"]);
    assert_eq!(row.candidate_edge_kinds, vec!["depends_on", "depends_on"]);
    assert_f64_eq(row.keyword_score, 1.0);
}

#[test]
fn build_graph_structural_generic_topology_candidate_inputs_composes() {
    let candidate = build_graph_structural_generic_topology_candidate_inputs(
        build_graph_structural_generic_topology_candidate_metadata_inputs(
            "candidate-chain",
            vec![
                "node-1".to_string(),
                "node-2".to_string(),
                "node-3".to_string(),
            ],
            vec!["node-1".to_string(), "node-2".to_string()],
            vec!["node-2".to_string(), "node-3".to_string()],
            vec!["depends_on".to_string(), "depends_on".to_string()],
        ),
        0.6,
        0.3,
        0.2,
        0.1,
    );

    assert_eq!(
        candidate,
        GraphStructuralGenericTopologyCandidateInputs::new(
            GraphStructuralGenericTopologyCandidateMetadataInputs::new(
                "candidate-chain",
                vec![
                    "node-1".to_string(),
                    "node-2".to_string(),
                    "node-3".to_string(),
                ],
                vec!["node-1".to_string(), "node-2".to_string()],
                vec!["node-2".to_string(), "node-3".to_string()],
                vec!["depends_on".to_string(), "depends_on".to_string()],
            ),
            0.6,
            0.3,
            0.2,
            0.1,
        )
    );
}
