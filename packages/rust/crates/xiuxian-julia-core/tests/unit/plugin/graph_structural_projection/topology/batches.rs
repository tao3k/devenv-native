use super::{
    Float64Array, GraphStructuralQueryAnchor, GraphStructuralQueryContext, OptionTestExt,
    ResultTestExt, StringArray, build_graph_structural_generic_topology_candidate_inputs,
    build_graph_structural_generic_topology_candidate_metadata_inputs,
    build_graph_structural_generic_topology_rerank_request_batch,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_keyword_tag_query_context,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples,
};
#[test]
fn build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections_composes()
 {
    let batch =
            build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections(
                &build_graph_structural_keyword_tag_query_context(
                    "query-generic-raw-connected-collections",
                    0,
                    2,
                    vec!["alpha".to_string()],
                    Vec::new(),
                    vec!["related".to_string()],
                )
                .or_panic("query context"),
                &[build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
                    "candidate-from-raw-connected-collection",
                    vec![
                        ("node-1", "node-2", 0.4),
                        ("node-2", "node-3", 0.8),
                    ],
                    "related",
                    0.3,
                    1.0,
                    0.0,
                )
                .or_panic("raw connected pair collection candidate")],
            )
            .or_panic("raw connected pair collection batch should project");

    assert_eq!(batch.num_rows(), 1);
    let candidate_ids = batch
        .column_by_name("candidate_id")
        .or_panic("candidate_id column")
        .as_any()
        .downcast_ref::<StringArray>()
        .or_panic("candidate_id strings");
    let semantic_scores = batch
        .column_by_name("semantic_score")
        .or_panic("semantic_score column")
        .as_any()
        .downcast_ref::<Float64Array>()
        .or_panic("semantic_score floats");

    assert_eq!(
        candidate_ids.value(0),
        "candidate-from-raw-connected-collection"
    );
    assert!((semantic_scores.value(0) - 0.6).abs() < f64::EPSILON);
}

#[test]
fn build_graph_structural_generic_topology_rerank_request_batch_composes() {
    let query = GraphStructuralQueryContext::new(
        "query-generic-batch",
        0,
        2,
        vec![
            GraphStructuralQueryAnchor::new("semantic", "symbol:entry").or_panic("semantic anchor"),
        ],
        vec!["depends_on".to_string()],
    )
    .or_panic("query context");
    let batch = build_graph_structural_generic_topology_rerank_request_batch(
        &query,
        &[build_graph_structural_generic_topology_candidate_inputs(
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
            0.8,
            0.4,
            0.2,
            0.1,
        )],
    )
    .or_panic("generic topology batch helper should normalize");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 15);
}
