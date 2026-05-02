use super::{
    GraphStructuralKeywordOverlapCandidateInputs, GraphStructuralKeywordOverlapQueryInputs,
    GraphStructuralKeywordOverlapRawCandidateInputs, GraphStructuralNodeMetadataInputs,
    GraphStructuralPairCandidateInputs, assert_f64_eq,
    build_graph_structural_keyword_overlap_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_pair_request_input,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
};
#[test]
fn build_graph_structural_keyword_overlap_pair_request_input_composes() {
    let request = build_graph_structural_keyword_overlap_pair_request_input(
        &build_graph_structural_keyword_overlap_query_inputs(
            "query-12",
            1,
            2,
            vec!["alpha".to_string()],
            vec!["semantic_similar".to_string()],
        ),
        GraphStructuralKeywordOverlapCandidateInputs::new(
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string()]),
            GraphStructuralNodeMetadataInputs::new(vec!["core".to_string()]),
            GraphStructuralPairCandidateInputs::new(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
            ),
            0.6,
            0.2,
            true,
        ),
    );

    assert_eq!(request.metadata_inputs.query_inputs.query_id, "query-12");
    assert_eq!(
        request.metadata_inputs.query_inputs.keyword_anchors,
        vec!["alpha"]
    );
    assert_eq!(request.metadata_inputs.pair_inputs.left_id, "node-left");
    assert_eq!(request.metadata_inputs.pair_inputs.right_id, "node-right");
    assert_f64_eq(request.semantic_score, 0.6);
}

#[test]
fn build_graph_structural_keyword_overlap_query_inputs_composes() {
    let query = build_graph_structural_keyword_overlap_query_inputs(
        "query-12b",
        1,
        2,
        vec!["alpha".to_string()],
        vec!["semantic_similar".to_string()],
    );

    assert_eq!(
        query,
        GraphStructuralKeywordOverlapQueryInputs::new(
            "query-12b",
            1,
            2,
            vec!["alpha".to_string()],
            vec!["semantic_similar".to_string()],
        )
    );
}

#[test]
fn build_graph_structural_keyword_overlap_pair_candidate_inputs_composes() {
    let candidate = build_graph_structural_keyword_overlap_pair_candidate_inputs(
        build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
            "node-left",
            "node-right",
            vec!["semantic_similar".to_string()],
            vec!["alpha".to_string(), "core".to_string()],
            vec!["core".to_string()],
        ),
        0.6,
        0.2,
        true,
    );

    assert_eq!(
        candidate,
        GraphStructuralKeywordOverlapCandidateInputs::new(
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string(),]),
            GraphStructuralNodeMetadataInputs::new(vec!["core".to_string()]),
            GraphStructuralPairCandidateInputs::new(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
            ),
            0.6,
            0.2,
            true,
        )
    );
}

#[test]
fn build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw_composes() {
    let candidate = build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw(
        build_graph_structural_keyword_overlap_raw_candidate_inputs(
            build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
                vec!["alpha".to_string(), "core".to_string()],
                vec!["core".to_string()],
            ),
            0.6,
            0.2,
            true,
        ),
    );

    assert_eq!(
        candidate,
        GraphStructuralKeywordOverlapCandidateInputs::new(
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string(),]),
            GraphStructuralNodeMetadataInputs::new(vec!["core".to_string()]),
            GraphStructuralPairCandidateInputs::new(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
            ),
            0.6,
            0.2,
            true,
        )
    );
}

#[test]
fn build_graph_structural_keyword_overlap_raw_candidate_inputs_composes() {
    let candidate = build_graph_structural_keyword_overlap_raw_candidate_inputs(
        build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
            "node-left",
            "node-right",
            vec!["semantic_similar".to_string()],
            vec!["alpha".to_string(), "core".to_string()],
            vec!["core".to_string()],
        ),
        0.6,
        0.2,
        true,
    );

    assert_eq!(
        candidate,
        GraphStructuralKeywordOverlapRawCandidateInputs::new(
            build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
                vec!["alpha".to_string(), "core".to_string()],
                vec!["core".to_string()],
            ),
            0.6,
            0.2,
            true,
        )
    );
}

#[test]
fn build_graph_structural_keyword_overlap_candidate_inputs_composes() {
    let candidate = build_graph_structural_keyword_overlap_candidate_inputs(
        build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
            "node-left",
            "node-right",
            vec!["semantic_similar".to_string()],
            vec!["alpha".to_string(), "core".to_string()],
            vec!["core".to_string()],
        ),
        0.6,
        0.2,
        true,
    );

    assert_eq!(
        candidate,
        GraphStructuralKeywordOverlapCandidateInputs::new(
            GraphStructuralNodeMetadataInputs::new(vec!["alpha".to_string(), "core".to_string(),]),
            GraphStructuralNodeMetadataInputs::new(vec!["core".to_string()]),
            GraphStructuralPairCandidateInputs::new(
                "node-left",
                "node-right",
                vec!["semantic_similar".to_string()],
            ),
            0.6,
            0.2,
            true,
        )
    );
}
