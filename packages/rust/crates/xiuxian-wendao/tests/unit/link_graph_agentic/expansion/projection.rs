use super::support::{
    Float64Array, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN, Int32Array, ListArray, RegisteredRepository,
    RepositoryPluginConfig, RepositoryRefreshPolicy, StringArray, TestResult,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs, build_index_fixture,
    build_pair_rerank_request_batch, expansion_config,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    first_worker_pair, required_column,
};

#[test]
fn test_agentic_expansion_pair_projects_into_julia_graph_structural_request() -> TestResult {
    let fixture = build_index_fixture(&[
        (
            "notes/a.md",
            "---\ntags:\n  - alpha\n  - core\n---\n# A\n\nalpha momentum\n",
        ),
        (
            "notes/b.md",
            "---\ntags:\n  - core\n---\n# B\n\nalpha breakout\n",
        ),
        (
            "notes/c.md",
            "---\ntags:\n  - beta\n---\n# C\n\nbeta mean reversion\n",
        ),
        (
            "notes/d.md",
            "---\ntags:\n  - gamma\n---\n# D\n\ngamma divergence\n",
        ),
    ])?;
    let index = &fixture.index;
    let plan = index.agentic_expansion_plan_with_config(Some("alpha"), expansion_config(2, 4, 2));

    let pair = first_worker_pair(&plan);
    let batch = build_pair_rerank_request_batch(index, pair)?;

    let query_ids =
        required_column::<StringArray>(&batch, GRAPH_STRUCTURAL_QUERY_ID_COLUMN, "utf8");
    let retrieval_layers =
        required_column::<Int32Array>(&batch, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, "int32");
    let query_max_layers =
        required_column::<Int32Array>(&batch, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, "int32");
    let semantic_scores =
        required_column::<Float64Array>(&batch, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN, "float64");
    let keyword_scores =
        required_column::<Float64Array>(&batch, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN, "float64");
    let tag_scores =
        required_column::<Float64Array>(&batch, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN, "float64");
    let anchor_planes =
        required_column::<ListArray>(&batch, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, "list");
    let anchor_values =
        required_column::<ListArray>(&batch, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, "list");
    let candidate_node_ids =
        required_column::<ListArray>(&batch, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN, "list");
    let candidate_edge_sources = required_column::<ListArray>(
        &batch,
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
        "list",
    );
    let candidate_edge_destinations = required_column::<ListArray>(
        &batch,
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
        "list",
    );
    let candidate_edge_kinds =
        required_column::<ListArray>(&batch, GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, "list");

    let anchor_plane_values = anchor_planes.value(0);
    let Some(anchor_plane_values) = anchor_plane_values.as_any().downcast_ref::<StringArray>()
    else {
        panic!("anchor plane values should be utf8");
    };
    let anchor_value_values = anchor_values.value(0);
    let Some(anchor_value_values) = anchor_value_values.as_any().downcast_ref::<StringArray>()
    else {
        panic!("anchor values should be utf8");
    };
    let candidate_node_values = candidate_node_ids.value(0);
    let Some(candidate_node_values) = candidate_node_values.as_any().downcast_ref::<StringArray>()
    else {
        panic!("candidate node ids should be utf8");
    };

    assert_eq!(query_ids.value(0), "agentic-query-alpha");
    assert_eq!(retrieval_layers.value(0), 0);
    assert_eq!(query_max_layers.value(0), 1);
    assert_eq!(anchor_plane_values.value(0), "keyword");
    assert_eq!(anchor_value_values.value(0), "alpha");
    assert_eq!(candidate_node_ids.value_length(0), 2);
    assert_eq!(candidate_edge_sources.value_length(0), 0);
    assert_eq!(candidate_edge_destinations.value_length(0), 0);
    assert_eq!(candidate_edge_kinds.value_length(0), 0);
    assert_eq!(candidate_node_values.value(0), pair.left_id);
    assert!(semantic_scores.value(0) > 0.0);
    assert!((keyword_scores.value(0) - 1.0).abs() < f64::EPSILON);
    assert!((tag_scores.value(0) - 1.0).abs() < f64::EPSILON);
    assert_eq!(batch.num_rows(), 1);

    Ok(())
}

#[tokio::test]
async fn test_agentic_expansion_pair_uses_julia_graph_structural_fetch_helper() -> TestResult {
    let fixture = build_index_fixture(&[
        (
            "notes/a.md",
            "---\ntags:\n  - alpha\n---\n# A\n\nalpha momentum\n",
        ),
        (
            "notes/b.md",
            "---\ntags:\n  - alpha\n---\n# B\n\nalpha breakout\n",
        ),
        (
            "notes/c.md",
            "---\ntags:\n  - beta\n---\n# C\n\nbeta mean reversion\n",
        ),
        (
            "notes/d.md",
            "---\ntags:\n  - gamma\n---\n# D\n\ngamma divergence\n",
        ),
    ])?;
    let index = &fixture.index;
    let plan = index.agentic_expansion_plan_with_config(Some("alpha"), expansion_config(2, 4, 2));

    let pair = first_worker_pair(&plan);
    let left = index
        .metadata(&pair.left_id)
        .ok_or_else(|| format!("missing metadata for `{}`", pair.left_id))?;
    let right = index
        .metadata(&pair.right_id)
        .ok_or_else(|| format!("missing metadata for `{}`", pair.right_id))?;
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({}),
        }],
    };

    let Err(error) =
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            &repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                "agentic-query-alpha",
                0,
                2,
                vec!["alpha".to_string()],
                vec!["depends_on".to_string()],
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    pair.left_id.clone(),
                    pair.right_id.clone(),
                    vec!["depends_on".to_string()],
                    left.tags.clone(),
                    right.tags.clone(),
                ),
                0.7,
                0.6,
                true,
            )],
        )
        .await
    else {
        panic!("missing graph-structural transport must fail");
    };

    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );

    Ok(())
}
