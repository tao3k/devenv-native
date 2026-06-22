use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use crate::{
    GraphStructuralFilterConstraint, GraphStructuralGenericTopologyCandidateInput,
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInput,
    GraphStructuralGenericTopologyCandidateMetadataInputs,
    GraphStructuralKeywordTagQueryContextInput,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    GraphStructuralRawConnectedPairCollectionRawTupleInput, JuliaContractKind,
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections,
    julia_plugin_test_support::common::ResultTestExt,
    julia_plugin_test_support::wendaosearch_services::{
        LIVE_REQUEST_TIMEOUT_SECS, LIVE_SERVICE_STARTUP_TIMEOUT_SECS, await_live_step,
        reserve_real_service_port, solver_demo_multi_route_base_url_for_port,
        solver_demo_wendaosearch_service_available,
        spawn_real_wendaosearch_solver_demo_multi_route_service,
        wait_for_service_ready_with_attempts,
    },
};

use super::{
    fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections,
};

fn build_graph_structural_keyword_tag_query_context(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    tag_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> Result<
    crate::GraphStructuralQueryContext,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    crate::build_graph_structural_keyword_tag_query_context(
        GraphStructuralKeywordTagQueryContextInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            tag_anchors,
            edge_constraint_kinds,
        },
    )
}

fn build_graph_structural_generic_topology_candidate_metadata_inputs(
    candidate_id: impl Into<String>,
    node_ids: Vec<String>,
    edge_sources: Vec<String>,
    edge_destinations: Vec<String>,
    edge_kinds: Vec<String>,
) -> GraphStructuralGenericTopologyCandidateMetadataInputs {
    crate::build_graph_structural_generic_topology_candidate_metadata_inputs(
        GraphStructuralGenericTopologyCandidateMetadataInput {
            candidate_id: candidate_id.into(),
            node_ids,
            edge_sources,
            edge_destinations,
            edge_kinds,
        },
    )
}

fn build_graph_structural_generic_topology_candidate_inputs(
    metadata: GraphStructuralGenericTopologyCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> GraphStructuralGenericTopologyCandidateInputs {
    crate::build_graph_structural_generic_topology_candidate_inputs(
        GraphStructuralGenericTopologyCandidateInput {
            metadata,
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples<I, L, R>(
    candidate_id: impl Into<String>,
    pair_candidates: I,
    fallback_edge_kind: impl Into<String>,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> Result<
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
>
where
    I: IntoIterator<Item = (L, R, f64)>,
    L: Into<String>,
    R: Into<String>,
{
    crate::build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
        GraphStructuralRawConnectedPairCollectionRawTupleInput {
            candidate_id: candidate_id.into(),
            pair_candidates,
            fallback_edge_kind: JuliaContractKind::from(fallback_edge_kind.into()),
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn assert_generic_topology_filter_row_accepted(
    row: &crate::GraphStructuralFilterScoreRow,
    candidate_id: &str,
    expected_pin_count: usize,
) {
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.accepted);
    assert!(row.structural_score > 0.0);
    assert_eq!(row.pin_assignment.len(), expected_pin_count);
    assert_eq!(row.rejection_reason, "");
}

#[tokio::test]
async fn fetch_graph_structural_generic_topology_rerank_rows_for_repository_rejects_missing_transport()
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

    let error = fetch_graph_structural_generic_topology_rerank_rows_for_repository(
        &repository,
        &build_graph_structural_keyword_tag_query_context(
            "query-generic",
            0,
            2,
            vec!["alpha".to_string()],
            Vec::new(),
            vec!["depends_on".to_string()],
        )
        .or_panic("generic topology query"),
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
            0.5,
            1.0,
            0.0,
        )],
    )
    .await
    .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

fn graph_structural_generic_topology_explicit_rerank_repository(
    base_url: &str,
) -> RegisteredRepository {
    RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": base_url,
                    "structural_rerank": {
                        "route": "/graph/structural/rerank",
                        "schema_version": "v0-draft",
                        "timeout_secs": LIVE_REQUEST_TIMEOUT_SECS
                    }
                }
            }),
        }],
    }
}

async fn assert_solver_demo_explicit_generic_topology_single_rerank(
    repository: &RegisteredRepository,
) {
    let candidate_id = "candidate-chain-live".to_string();
    let rows = await_live_step(
        fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections(
            repository,
            &build_graph_structural_keyword_tag_query_context(
                "query-live-generic",
                0,
                2,
                vec!["alpha".to_string()],
                Vec::new(),
                vec!["depends_on".to_string()],
            )
            .or_panic("generic topology query"),
            &[build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
                candidate_id.clone(),
                vec![("node-1", "node-2", 0.6), ("node-2", "node-3", 0.8)],
                "depends_on",
                0.6,
                1.0,
                0.0,
            )
            .or_panic("raw connected pair collection candidate")],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "real WendaoSearch solver-demo generic-topology rerank",
    )
    .await
        .unwrap_or_else(|error| {
            panic!("real WendaoSearch solver-demo generic-topology rerank should succeed: {error}")
        });

    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo generic live response")
    });
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.feasible);
    assert!(row.structural_score > 0.0);
    assert!(row.final_score > row.structural_score);
    assert_eq!(row.pin_assignment, vec!["node-1".to_string()]);
    assert!(
        row.explanation.contains("with 3 nodes, 2 explicit edges"),
        "unexpected explanation: {}",
        row.explanation
    );
}

fn graph_structural_generic_topology_manifest_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "capability_manifest_transport": {
                    "base_url": base_url,
                    "route": "/plugin/capabilities",
                    "schema_version": "v0-draft",
                    "timeout_secs": LIVE_REQUEST_TIMEOUT_SECS
                }
            }),
        }],
    }
}

async fn assert_solver_demo_multi_route_generic_topology_single_rerank(
    repository: &RegisteredRepository,
) {
    let candidate_id = "candidate-chain-live".to_string();
    let rows = await_live_step(
        fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections(
            repository,
            &build_graph_structural_keyword_tag_query_context(
                "query-live-generic",
                0,
                2,
                vec!["alpha".to_string()],
                Vec::new(),
                vec!["depends_on".to_string()],
            )
            .or_panic("generic topology query"),
            &[build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
                candidate_id.clone(),
                vec![("node-1", "node-2", 0.6), ("node-2", "node-3", 0.8)],
                "depends_on",
                0.6,
                1.0,
                0.0,
            )
            .or_panic("raw connected pair collection candidate")],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "manifest-discovered real WendaoSearch solver-demo generic-topology rerank",
    )
    .await
        .unwrap_or_else(|error| {
            panic!(
                "manifest-discovered real WendaoSearch solver-demo generic-topology rerank should succeed: {error}"
            )
        });

    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo generic multi-route response")
    });
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.feasible);
    assert!(row.structural_score > 0.0);
    assert!(row.final_score > row.structural_score);
    assert_eq!(row.pin_assignment, vec!["node-1".to_string()]);
    assert!(
        row.explanation.contains("with 3 nodes, 2 explicit edges"),
        "unexpected explanation: {}",
        row.explanation
    );
}

async fn assert_solver_demo_multi_route_generic_topology_multi_rerank(
    repository: &RegisteredRepository,
) {
    let rows = await_live_step(
        fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections(
            repository,
            &build_graph_structural_keyword_tag_query_context(
                "query-live-generic-batch",
                0,
                2,
                vec!["alpha".to_string()],
                Vec::new(),
                vec!["depends_on".to_string()],
            )
            .or_panic("generic topology query"),
            &[
                build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
                    "candidate-chain-live-a",
                    vec![("node-1", "node-2", 0.6), ("node-2", "node-3", 0.8)],
                    "depends_on",
                    0.6,
                    1.0,
                    0.0,
                )
                .or_panic("raw connected pair collection candidate"),
                build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
                    "candidate-chain-live-b",
                    vec![("node-4", "node-5", 0.55), ("node-5", "node-6", 0.75)],
                    "depends_on",
                    0.5,
                    1.0,
                    0.0,
                )
                .or_panic("raw connected pair collection candidate"),
            ],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "manifest-discovered real WendaoSearch solver-demo multi-candidate generic-topology rerank",
    )
    .await
        .unwrap_or_else(|error| {
            panic!(
                "manifest-discovered real WendaoSearch solver-demo multi-candidate generic-topology rerank should succeed: {error}"
            )
        });

    assert_eq!(rows.len(), 2);
    for candidate_id in ["candidate-chain-live-a", "candidate-chain-live-b"] {
        let row = rows.get(candidate_id).unwrap_or_else(|| {
            panic!("missing candidate `{candidate_id}` in solver-demo generic multi-route response")
        });
        assert_eq!(row.candidate_id, candidate_id);
        assert!(row.feasible);
        assert!(row.structural_score > 0.0);
        assert!(row.final_score > row.structural_score);
        assert_eq!(row.pin_assignment.len(), 1);
        assert!(
            row.explanation.contains("with 3 nodes, 2 explicit edges"),
            "unexpected explanation for `{candidate_id}`: {}",
            row.explanation
        );
    }
}

#[tokio::test]
async fn fetch_graph_structural_generic_topology_filter_rows_for_repository_rejects_missing_transport()
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

    let query = build_graph_structural_keyword_tag_query_context(
        "query-generic-filter",
        0,
        2,
        vec!["alpha".to_string()],
        Vec::new(),
        vec!["depends_on".to_string()],
    )
    .or_panic("generic topology filter query");
    let constraint =
        GraphStructuralFilterConstraint::new("pin_assignment", 1).or_panic("filter constraint");
    let candidates = [
        build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
            "candidate-chain-filter",
            vec![("node-1", "node-2", 0.6), ("node-2", "node-3", 0.8)],
            "depends_on",
            0.6,
            1.0,
            0.0,
        )
        .or_panic("raw connected pair collection candidate"),
    ];

    let error = fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections(
        &repository,
        &query,
        &constraint,
        &candidates,
    )
    .await
    .err_or_panic("missing graph-structural filter transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/filter"),
        "unexpected error: {error}"
    );
}

async fn assert_solver_demo_multi_route_generic_topology_single_filter(
    repository: &RegisteredRepository,
) {
    let query = build_graph_structural_keyword_tag_query_context(
        "query-live-generic-filter",
        0,
        2,
        vec!["alpha".to_string()],
        vec!["alpha".to_string()],
        vec!["depends_on".to_string()],
    )
    .or_panic("generic topology filter query");
    let constraint =
        GraphStructuralFilterConstraint::new("boundary_match", 2).or_panic("filter constraint");
    let candidate_id = "candidate-chain-filter-live".to_string();
    let candidates = [
        build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
            candidate_id.clone(),
            vec![("node-1", "node-2", 0.6), ("node-2", "node-3", 0.8)],
            "depends_on",
            0.6,
            1.0,
            0.0,
        )
        .or_panic("raw connected pair collection candidate"),
    ];

    let request_batch =
        build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections(
            &query,
            &constraint,
            &candidates,
        )
        .or_panic("generic topology filter request batch");
    assert_eq!(request_batch.num_rows(), 1);

    let rows = await_live_step(
        fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections(
            repository,
            &query,
            &constraint,
            &candidates,
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "manifest-discovered real WendaoSearch solver-demo generic-topology filter",
    )
    .await
    .unwrap_or_else(|error| {
        panic!(
            "manifest-discovered real WendaoSearch solver-demo generic-topology filter should succeed: {error}"
        )
    });

    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo generic filter response")
    });
    assert_generic_topology_filter_row_accepted(row, &candidate_id, 2);
}

async fn assert_solver_demo_multi_route_generic_topology_multi_filter(
    repository: &RegisteredRepository,
) {
    let query = build_graph_structural_keyword_tag_query_context(
        "query-live-generic-filter-batch",
        0,
        2,
        vec!["alpha".to_string()],
        vec!["alpha".to_string()],
        vec!["depends_on".to_string()],
    )
    .or_panic("generic topology filter query");
    let constraint =
        GraphStructuralFilterConstraint::new("boundary_match", 2).or_panic("filter constraint");
    let candidates = [
        build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
            "candidate-chain-filter-live-a",
            vec![("node-1", "node-2", 0.6), ("node-2", "node-3", 0.8)],
            "depends_on",
            0.6,
            1.0,
            0.0,
        )
        .or_panic("raw connected pair collection candidate"),
        build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
            "candidate-chain-filter-live-b",
            vec![("node-4", "node-5", 0.55), ("node-5", "node-6", 0.75)],
            "depends_on",
            0.5,
            1.0,
            0.0,
        )
        .or_panic("raw connected pair collection candidate"),
    ];

    let request_batch =
        build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections(
            &query,
            &constraint,
            &candidates,
        )
        .or_panic("generic topology multi-candidate filter request batch");
    assert_eq!(request_batch.num_rows(), 2);

    let rows = await_live_step(
        fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections(
            repository,
            &query,
            &constraint,
            &candidates,
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "manifest-discovered real WendaoSearch solver-demo multi-candidate generic-topology filter",
    )
    .await
    .unwrap_or_else(|error| {
        panic!(
            "manifest-discovered real WendaoSearch solver-demo multi-candidate generic-topology filter should succeed: {error}"
        )
    });

    assert_eq!(rows.len(), 2);
    for candidate_id in [
        "candidate-chain-filter-live-a",
        "candidate-chain-filter-live-b",
    ] {
        let row = rows.get(candidate_id).unwrap_or_else(|| {
            panic!("missing candidate `{candidate_id}` in solver-demo generic filter response")
        });
        assert_generic_topology_filter_row_accepted(row, candidate_id, 2);
    }
}

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
async fn fetch_graph_structural_generic_topology_rows_for_repository_via_manifest_discovery_against_real_wendaosearch_solver_demo_multi_route_service()
 {
    if !solver_demo_wendaosearch_service_available() {
        eprintln!(
            "skipping real WendaoSearch solver-demo generic topology service test; set WENDAOSEARCH_SOLVER_DEMO_BASE_URL or WENDAOSEARCH_PACKAGE_DIR"
        );
        return;
    }

    let port = reserve_real_service_port();
    let base_url = solver_demo_multi_route_base_url_for_port(port);
    let mut service = spawn_real_wendaosearch_solver_demo_multi_route_service(port);
    let explicit_repository =
        graph_structural_generic_topology_explicit_rerank_repository(&base_url);
    let manifest_repository = graph_structural_generic_topology_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&base_url, 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch solver-demo multi-route Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for real WendaoSearch solver-demo multi-route Flight service: {error}")
    });

    assert_solver_demo_explicit_generic_topology_single_rerank(&explicit_repository).await;
    assert_solver_demo_multi_route_generic_topology_single_rerank(&manifest_repository).await;
    assert_solver_demo_multi_route_generic_topology_multi_rerank(&manifest_repository).await;
    assert_solver_demo_multi_route_generic_topology_single_filter(&manifest_repository).await;
    assert_solver_demo_multi_route_generic_topology_multi_filter(&manifest_repository).await;
    service.kill();
}
