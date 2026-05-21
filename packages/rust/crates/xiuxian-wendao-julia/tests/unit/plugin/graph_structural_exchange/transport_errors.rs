use crate::{
    GraphStructuralFilterRequestRow,
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
    julia_plugin_test_support::common::ResultTestExt,
};
use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use super::{
    GraphStructuralRerankRequestRow, build_graph_structural_filter_request_batch,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    build_graph_structural_rerank_request_batch, fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    fetch_graph_structural_rerank_rows_for_repository,
};

#[tokio::test]
async fn fetch_graph_structural_rerank_rows_for_repository_rejects_missing_transport() {
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

    let error = fetch_graph_structural_rerank_rows_for_repository(&repository, &[batch])
        .await
        .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_rejects_missing_transport()
 {
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

    let error = fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository(
        &repository,
        &build_graph_structural_keyword_overlap_query_inputs(
            "query-1",
            0,
            2,
            vec!["alpha".to_string()],
            vec!["depends_on".to_string()],
        ),
        &[
            build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw(
                build_graph_structural_keyword_overlap_raw_candidate_inputs(
                    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                        "node-1",
                        "node-2",
                        vec!["depends_on".to_string()],
                        vec!["alpha".to_string(), "core".to_string()],
                        vec!["core".to_string()],
                    ),
                    0.7,
                    0.6,
                    true,
                ),
            ),
        ],
    )
    .await
    .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates_rejects_missing_transport()
 {
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

    let error =
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            &repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                "query-raw",
                0,
                2,
                vec!["alpha".to_string()],
                vec!["depends_on".to_string()],
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string()],
                ),
                0.7,
                0.6,
                true,
            )],
        )
        .await
        .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn fetch_graph_structural_filter_rows_for_repository_rejects_missing_transport() {
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

    let batch = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 1,
        query_max_layers: 3,
        constraint_kind: "boundary-match".into(),
        required_boundary_size: 2,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("filter request batch");

    let error = fetch_graph_structural_filter_rows_for_repository(&repository, &[batch])
        .await
        .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/filter"),
        "unexpected error: {error}"
    );
}
