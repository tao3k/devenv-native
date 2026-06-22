use super::{
    GraphStructuralKeywordOverlapPairInputs, GraphStructuralKeywordOverlapPairRequestInputs,
    GraphStructuralKeywordOverlapPairRerankInputs, GraphStructuralKeywordTagQueryInputs,
    GraphStructuralNodeMetadataInputs, GraphStructuralPairCandidateInputs, ResultTestExt,
    build_graph_structural_keyword_overlap_pair_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
};
#[test]
fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata_composes() {
    let batch = build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata(&[
        GraphStructuralKeywordOverlapPairRerankInputs::new(
            GraphStructuralKeywordOverlapPairInputs::new(
                GraphStructuralKeywordTagQueryInputs::new(
                    "query-10",
                    1,
                    2,
                    vec!["alpha".to_string()],
                    Vec::new(),
                    Vec::new(),
                ),
                GraphStructuralNodeMetadataInputs::new(vec![
                    "alpha".to_string(),
                    "core".to_string(),
                ]),
                GraphStructuralNodeMetadataInputs::new(vec![
                    "graph".to_string(),
                    "core".to_string(),
                ]),
                GraphStructuralPairCandidateInputs::new("node-r", "node-a", Vec::new()),
            ),
            0.5,
            0.0,
            true,
        ),
    ])
    .or_panic("metadata-aware batch helper should normalize");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 15);
}

#[test]
fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs_composes() {
    let batch = build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs(&[
        GraphStructuralKeywordOverlapPairRequestInputs::new(
            GraphStructuralKeywordTagQueryInputs::new(
                "query-11",
                0,
                1,
                vec!["alpha".to_string()],
                Vec::new(),
                Vec::new(),
            ),
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string()]),
            GraphStructuralNodeMetadataInputs::new(vec!["core".to_string(), "graph".to_string()]),
            GraphStructuralPairCandidateInputs::new("node-left", "node-right", Vec::new()),
            0.7,
            0.0,
            true,
        ),
    ])
    .or_panic("higher-level candidate input helper should normalize");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 15);
}

#[test]
fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates_composes() {
    let batch =
        build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(
            &build_graph_structural_keyword_overlap_query_inputs(
                "query-13a",
                0,
                1,
                vec!["alpha".to_string()],
                Vec::new(),
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-left",
                    "node-right",
                    Vec::new(),
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string(), "graph".to_string()],
                ),
                0.7,
                0.0,
                true,
            )],
        )
        .or_panic("raw candidate batch helper should normalize");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 15);
}

#[test]
fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_composes() {
    let batch = build_graph_structural_keyword_overlap_pair_rerank_request_batch(
        &build_graph_structural_keyword_overlap_query_inputs(
            "query-13",
            0,
            1,
            vec!["alpha".to_string()],
            Vec::new(),
        ),
        &[
            build_graph_structural_keyword_overlap_pair_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-left",
                    "node-right",
                    Vec::new(),
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string(), "graph".to_string()],
                ),
                0.7,
                0.0,
                true,
            ),
        ],
    )
    .or_panic("query-candidate batch helper should normalize");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 15);
}
