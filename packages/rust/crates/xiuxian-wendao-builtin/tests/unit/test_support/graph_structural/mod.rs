use xiuxian_julia_core::{
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN as JULIA_GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN as JULIA_GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN as JULIA_GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN as JULIA_GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN as JULIA_GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN as JULIA_GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN as JULIA_GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN as JULIA_GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN as JULIA_GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN as JULIA_GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN as JULIA_GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN as JULIA_GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    GraphStructuralKeywordOverlapCandidateMetadataInput, GraphStructuralKeywordOverlapQueryInput,
    GraphStructuralKeywordOverlapRawCandidateInput,
};

use crate::test_support::{
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
};

#[test]
fn linked_builtin_graph_structural_constants_match_julia_plugin_constants() {
    assert_eq!(
        GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
        JULIA_GRAPH_STRUCTURAL_QUERY_ID_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
        JULIA_GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
        JULIA_GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
        JULIA_GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
        JULIA_GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
        JULIA_GRAPH_STRUCTURAL_TAG_SCORE_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
        JULIA_GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
        JULIA_GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
        JULIA_GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
        JULIA_GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
        JULIA_GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN
    );
    assert_eq!(
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
        JULIA_GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN
    );
}

#[test]
fn linked_builtin_graph_structural_request_builders_compose() {
    let batch =
        build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(
            &build_graph_structural_keyword_overlap_query_inputs(
                GraphStructuralKeywordOverlapQueryInput {
                    query_id: "agentic-query-alpha".to_string(),
                    retrieval_layer: 0,
                    query_max_layers: 1,
                    keyword_anchors: vec!["alpha".to_string()],
                    edge_constraint_kinds: Vec::new(),
                },
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                GraphStructuralKeywordOverlapRawCandidateInput {
                    metadata_inputs:
                        build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                            GraphStructuralKeywordOverlapCandidateMetadataInput {
                                left_id: "left".to_string(),
                                right_id: "right".to_string(),
                                edge_kinds: Vec::new(),
                                left_tags: vec!["alpha".to_string()],
                                right_tags: vec!["alpha".to_string()],
                            },
                        ),
                    semantic_score: 0.7,
                    dependency_score: 0.0,
                    keyword_match: true,
                },
            )],
        )
        .unwrap_or_else(|error| {
            panic!("linked builtin graph-structural helpers should compose: {error}")
        });

    assert_eq!(batch.num_rows(), 1);
    assert!(
        batch
            .column_by_name(GRAPH_STRUCTURAL_QUERY_ID_COLUMN)
            .is_some()
    );
    assert!(
        batch
            .column_by_name(GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN)
            .is_some()
    );
    assert!(
        batch
            .column_by_name(GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN)
            .is_some()
    );
    assert!(
        batch
            .column_by_name(GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN)
            .is_some()
    );
}
