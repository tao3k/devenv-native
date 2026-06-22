use super::{
    ResultTestExt, assert_f64_eq,
    build_graph_structural_generic_topology_candidate_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs,
    build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection,
    build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_subgraph,
    build_graph_structural_generic_topology_rerank_request_row,
    build_graph_structural_keyword_tag_query_context, build_graph_structural_pair_candidate_inputs,
    build_graph_structural_raw_connected_pair_inputs,
    build_graph_structural_scored_pair_candidate_inputs,
};
#[test]
fn build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection_projects_edges()
 {
    let candidate = build_graph_structural_generic_topology_candidate_subgraph(
        build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection(
            "candidate-from-pairs",
            vec![
                build_graph_structural_pair_candidate_inputs("node-1", "node-2", Vec::new()),
                build_graph_structural_pair_candidate_inputs(
                    "node-2",
                    "node-3",
                    vec!["references".to_string()],
                ),
            ],
            "related",
        )
        .or_panic("pair collection metadata should normalize"),
    )
    .or_panic("pair collection candidate should normalize");

    assert_eq!(candidate.candidate_id(), "candidate-from-pairs");
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
        &["related".to_string(), "references".to_string()]
    );
}

#[test]
fn build_graph_structural_generic_topology_candidate_inputs_from_pair_collection_preserves_scores()
{
    let row = build_graph_structural_generic_topology_rerank_request_row(
        &build_graph_structural_keyword_tag_query_context(
            "query-generic-pairs",
            0,
            2,
            vec!["alpha".to_string()],
            Vec::new(),
            vec!["depends_on".to_string()],
        )
        .or_panic("query context"),
        build_graph_structural_generic_topology_candidate_inputs_from_pair_collection(
            "candidate-from-pairs",
            vec![
                build_graph_structural_pair_candidate_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                ),
                build_graph_structural_pair_candidate_inputs("node-2", "node-3", Vec::new()),
            ],
            "related",
            0.7,
            0.6,
            1.0,
            0.0,
        )
        .or_panic("pair collection candidate should normalize"),
    )
    .or_panic("pair collection rerank row should project");

    assert_f64_eq(row.semantic_score, 0.7);
    assert_f64_eq(row.dependency_score, 0.6);
    assert_f64_eq(row.keyword_score, 1.0);
    assert_f64_eq(row.tag_score, 0.0);
    assert_eq!(
        row.candidate_edge_kinds,
        vec!["depends_on".to_string(), "related".to_string()]
    );
}

#[test]
fn build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection_averages_semantic_score()
 {
    let row = build_graph_structural_generic_topology_rerank_request_row(
        &build_graph_structural_keyword_tag_query_context(
            "query-generic-scored-pairs",
            0,
            2,
            vec!["alpha".to_string()],
            Vec::new(),
            vec!["depends_on".to_string()],
        )
        .or_panic("query context"),
        build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection(
            "candidate-from-scored-pairs",
            vec![
                build_graph_structural_scored_pair_candidate_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                    0.6,
                )
                .or_panic("scored pair candidate"),
                build_graph_structural_scored_pair_candidate_inputs(
                    "node-2",
                    "node-3",
                    Vec::new(),
                    0.8,
                )
                .or_panic("scored pair candidate"),
            ],
            "related",
            0.5,
            1.0,
            0.0,
        )
        .or_panic("scored pair collection candidate should normalize"),
    )
    .or_panic("scored pair collection rerank row should project");

    assert!((row.semantic_score - 0.7).abs() < f64::EPSILON);
    assert_f64_eq(row.dependency_score, 0.5);
    assert_f64_eq(row.keyword_score, 1.0);
    assert_f64_eq(row.tag_score, 0.0);
    assert_eq!(
        row.candidate_edge_kinds,
        vec!["depends_on".to_string(), "related".to_string()]
    );
}

#[test]
fn build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs_averages_semantic_score()
 {
    let row = build_graph_structural_generic_topology_rerank_request_row(
        &build_graph_structural_keyword_tag_query_context(
            "query-generic-raw-connected-pairs",
            0,
            2,
            vec!["alpha".to_string()],
            Vec::new(),
            vec!["related".to_string()],
        )
        .or_panic("query context"),
        build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs(
            "candidate-from-raw-connected-pairs",
            vec![
                build_graph_structural_raw_connected_pair_inputs("node-1", "node-2", 0.4)
                    .or_panic("raw connected pair"),
                build_graph_structural_raw_connected_pair_inputs("node-2", "node-3", 0.8)
                    .or_panic("raw connected pair"),
            ],
            "related",
            0.3,
            1.0,
            0.0,
        )
        .or_panic("raw connected pair collection candidate should normalize"),
    )
    .or_panic("raw connected pair collection rerank row should project");

    assert!((row.semantic_score - 0.6).abs() < f64::EPSILON);
    assert_f64_eq(row.dependency_score, 0.3);
    assert_f64_eq(row.keyword_score, 1.0);
    assert_f64_eq(row.tag_score, 0.0);
    assert_eq!(
        row.candidate_edge_kinds,
        vec!["related".to_string(), "related".to_string()]
    );
}
