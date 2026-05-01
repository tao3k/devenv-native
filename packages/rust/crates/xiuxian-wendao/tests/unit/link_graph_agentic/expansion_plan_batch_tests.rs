#[cfg(feature = "julia")]
use arrow::array::{Float64Array, Int32Array, StringArray};
#[cfg(feature = "julia")]
use xiuxian_wendao_builtin::{
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
};

#[cfg(feature = "julia")]
use super::super::expansion_plan_batch_support::{
    build_plan_aware_generic_topology_filter_request_batch,
    build_plan_aware_generic_topology_rerank_request_batch,
    build_worker_partition_plan_aware_generic_topology_batch_fixture,
    fetch_plan_aware_generic_topology_filter_rows_via_manifest_discovery,
    fetch_plan_aware_generic_topology_rows_via_manifest_discovery,
};
#[cfg(feature = "julia")]
use super::super::expansion_support::{
    assert_solver_demo_generic_topology_row_basics,
    assert_solver_demo_generic_topology_row_infeasible,
    assert_solver_demo_generic_topology_row_shape, default_agentic_execution_relation_edge_kind,
    required_column, required_utf8_list_row_values,
};
#[cfg(feature = "julia")]
use super::support::linked_builtin_spawn_wendaosearch_solver_demo_multi_route_service;
#[cfg(feature = "julia")]
use super::{TestResult, build_index_fixture, expansion_config};

#[cfg(feature = "julia")]
#[tokio::test]
#[serial_test::serial(julia_live)]
async fn test_host_uses_julia_graph_structural_generic_topology_fetch_helper_for_plan_aware_worker_partition_batches_via_manifest_discovery_against_solver_demo_multi_route_service()
-> TestResult {
    let fixture = build_index_fixture(&[
        (
            "notes/a.md",
            "---\ntags:\n  - alpha\n  - core\n---\n# A\n\nalpha momentum\n",
        ),
        (
            "notes/b.md",
            "---\ntags:\n  - alpha\n  - branch\n---\n# B\n\nalpha breakout\n",
        ),
        (
            "notes/c.md",
            "---\ntags:\n  - alpha\n  - leaf\n---\n# C\n\nalpha continuation\n",
        ),
        (
            "notes/d.md",
            "---\ntags:\n  - alpha\n  - bridge\n---\n# D\n\nalpha rotation\n",
        ),
        (
            "notes/e.md",
            "---\ntags:\n  - alpha\n  - satellite\n---\n# E\n\nalpha expansion\n",
        ),
    ])?;
    let plan = fixture
        .index
        .agentic_expansion_plan_with_config(Some("alpha"), expansion_config(2, 10, 3));
    let relation_edge_kind = default_agentic_execution_relation_edge_kind();

    let batch_fixture = build_worker_partition_plan_aware_generic_topology_batch_fixture(
        &fixture.index,
        &plan,
        2,
        2,
        "query-live-generic-worker-plan-batch",
        "candidate-worker-plan-live",
        &relation_edge_kind,
    )?;
    assert_eq!(batch_fixture.keyword_anchors, vec!["alpha".to_string()]);
    assert_eq!(
        batch_fixture.edge_constraint_kinds,
        vec![relation_edge_kind.clone()]
    );
    assert!(
        batch_fixture.tag_anchors.iter().any(|tag| tag == "alpha"),
        "expected shared worker seed tags to include `alpha`, got {:?}",
        batch_fixture.tag_anchors
    );
    assert_eq!(batch_fixture.candidates.len(), 2);

    let batch = build_plan_aware_generic_topology_rerank_request_batch(&batch_fixture)?;
    let query_ids =
        required_column::<StringArray>(&batch, GRAPH_STRUCTURAL_QUERY_ID_COLUMN, "utf8");
    let retrieval_layers =
        required_column::<Int32Array>(&batch, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, "int32");
    let query_max_layers =
        required_column::<Int32Array>(&batch, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, "int32");
    let candidate_ids =
        required_column::<StringArray>(&batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, "utf8");
    let dependency_scores = required_column::<Float64Array>(
        &batch,
        GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
        "float64",
    );
    let semantic_scores =
        required_column::<Float64Array>(&batch, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN, "float64");
    let keyword_scores =
        required_column::<Float64Array>(&batch, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN, "float64");
    let tag_scores =
        required_column::<Float64Array>(&batch, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN, "float64");

    assert_eq!(batch.num_rows(), batch_fixture.candidates.len());
    for (row_index, candidate) in batch_fixture.candidates.iter().enumerate() {
        assert_eq!(query_ids.value(row_index), batch_fixture.query_id);
        assert_eq!(
            retrieval_layers.value(row_index),
            batch_fixture.expected_retrieval_layer
        );
        assert_eq!(
            query_max_layers.value(row_index),
            batch_fixture.expected_query_max_layers
        );
        assert_eq!(
            required_utf8_list_row_values(&batch, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, row_index),
            batch_fixture.anchor_planes
        );
        assert_eq!(
            required_utf8_list_row_values(&batch, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, row_index),
            batch_fixture.anchor_values
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
                row_index
            ),
            batch_fixture.edge_constraint_kinds
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
                row_index
            ),
            candidate.expected_candidate_node_ids
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
                row_index
            ),
            candidate.expected_candidate_edge_sources
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
                row_index
            ),
            candidate.expected_candidate_edge_destinations
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
                row_index
            ),
            candidate.expected_candidate_edge_kinds
        );
        assert_eq!(candidate_ids.value(row_index), candidate.candidate_id);
        assert!(
            (semantic_scores.value(row_index) - candidate.expected_semantic_score).abs()
                < f64::EPSILON,
            "unexpected semantic_score for `{}`: {} != {}",
            candidate.candidate_id,
            semantic_scores.value(row_index),
            candidate.expected_semantic_score
        );
        assert!(
            (dependency_scores.value(row_index) - candidate.expected_dependency_score).abs()
                < f64::EPSILON,
            "unexpected dependency_score for `{}`: {} != {}",
            candidate.candidate_id,
            dependency_scores.value(row_index),
            candidate.expected_dependency_score
        );
        assert!(
            (keyword_scores.value(row_index) - candidate.expected_keyword_score).abs()
                < f64::EPSILON,
            "unexpected keyword_score for `{}`: {} != {}",
            candidate.candidate_id,
            keyword_scores.value(row_index),
            candidate.expected_keyword_score
        );
        assert!(
            (tag_scores.value(row_index) - candidate.expected_tag_score).abs() < f64::EPSILON,
            "unexpected tag_score for `{}`: {} != {}",
            candidate.candidate_id,
            tag_scores.value(row_index),
            candidate.expected_tag_score
        );
    }

    let (server_base_url, mut server_guard) =
        linked_builtin_spawn_wendaosearch_solver_demo_multi_route_service().await;
    let rows = fetch_plan_aware_generic_topology_rows_via_manifest_discovery(
        server_base_url,
        &batch_fixture,
    )
    .await?;
    server_guard.kill();

    assert_eq!(rows.len(), batch_fixture.candidates.len());
    assert!(
        rows.values().any(|row| row.feasible),
        "expected at least one feasible plan-aware worker-partition row, got {:?}",
        rows.keys().collect::<Vec<_>>()
    );
    for fixture in &batch_fixture.candidates {
        let row = rows.get(&fixture.candidate_id).ok_or_else(|| {
            format!(
                "missing solver_demo plan-aware worker-partition generic-topology row `{}`",
                fixture.candidate_id
            )
        })?;
        if row.feasible {
            assert_solver_demo_generic_topology_row_basics(row, &fixture.candidate_id);
            assert_solver_demo_generic_topology_row_shape(
                row,
                &fixture.candidate_id,
                fixture.expected_nodes,
                fixture.expected_edges,
            );
            assert_eq!(
                row.pin_assignment.len(),
                1,
                "unexpected pin assignment for `{}`: {:?}",
                fixture.candidate_id,
                row.pin_assignment
            );
        } else {
            assert_solver_demo_generic_topology_row_infeasible(row, &fixture.candidate_id);
        }
    }

    Ok(())
}

#[cfg(feature = "julia")]
#[tokio::test]
#[serial_test::serial(julia_live)]
async fn test_host_uses_julia_graph_structural_generic_topology_fetch_helper_for_plan_aware_worker_partition_constraint_filter_batches_via_manifest_discovery_against_solver_demo_multi_route_service()
-> TestResult {
    let fixture = build_index_fixture(&[
        (
            "notes/a.md",
            "---\ntags:\n  - alpha\n  - core\n---\n# A\n\nalpha momentum\n",
        ),
        (
            "notes/b.md",
            "---\ntags:\n  - alpha\n  - branch\n---\n# B\n\nalpha breakout\n",
        ),
        (
            "notes/c.md",
            "---\ntags:\n  - alpha\n  - leaf\n---\n# C\n\nalpha continuation\n",
        ),
        (
            "notes/d.md",
            "---\ntags:\n  - alpha\n  - bridge\n---\n# D\n\nalpha rotation\n",
        ),
        (
            "notes/e.md",
            "---\ntags:\n  - alpha\n  - satellite\n---\n# E\n\nalpha expansion\n",
        ),
    ])?;
    let plan = fixture
        .index
        .agentic_expansion_plan_with_config(Some("alpha"), expansion_config(2, 10, 3));
    let relation_edge_kind = default_agentic_execution_relation_edge_kind();

    let batch_fixture = build_worker_partition_plan_aware_generic_topology_batch_fixture(
        &fixture.index,
        &plan,
        2,
        2,
        "query-live-generic-worker-plan-filter",
        "candidate-worker-plan-filter-live",
        &relation_edge_kind,
    )?;
    let constraint_kind = batch_fixture.expected_constraint_kind.as_str();
    let required_boundary_size = batch_fixture.expected_required_boundary_size;
    let expected_boundary_size = match usize::try_from(required_boundary_size) {
        Ok(size) => size,
        Err(error) => panic!("required boundary size should be non-negative: {error}"),
    };

    let batch = build_plan_aware_generic_topology_filter_request_batch(
        &batch_fixture,
        constraint_kind,
        required_boundary_size,
    )?;
    let query_ids =
        required_column::<StringArray>(&batch, GRAPH_STRUCTURAL_QUERY_ID_COLUMN, "utf8");
    let retrieval_layers =
        required_column::<Int32Array>(&batch, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, "int32");
    let query_max_layers =
        required_column::<Int32Array>(&batch, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, "int32");
    let candidate_ids =
        required_column::<StringArray>(&batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, "utf8");
    let constraint_kinds =
        required_column::<StringArray>(&batch, GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, "utf8");
    let required_boundary_sizes = required_column::<Int32Array>(
        &batch,
        GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN,
        "int32",
    );

    assert_eq!(batch.num_rows(), batch_fixture.candidates.len());
    for (row_index, candidate) in batch_fixture.candidates.iter().enumerate() {
        assert_eq!(query_ids.value(row_index), batch_fixture.query_id);
        assert_eq!(
            retrieval_layers.value(row_index),
            batch_fixture.expected_retrieval_layer
        );
        assert_eq!(
            query_max_layers.value(row_index),
            batch_fixture.expected_query_max_layers
        );
        assert_eq!(
            required_utf8_list_row_values(&batch, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, row_index),
            batch_fixture.anchor_planes
        );
        assert_eq!(
            required_utf8_list_row_values(&batch, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, row_index),
            batch_fixture.anchor_values
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
                row_index
            ),
            batch_fixture.edge_constraint_kinds
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
                row_index
            ),
            candidate.expected_candidate_node_ids
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
                row_index
            ),
            candidate.expected_candidate_edge_sources
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
                row_index
            ),
            candidate.expected_candidate_edge_destinations
        );
        assert_eq!(
            required_utf8_list_row_values(
                &batch,
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
                row_index
            ),
            candidate.expected_candidate_edge_kinds
        );
        assert_eq!(candidate_ids.value(row_index), candidate.candidate_id);
        assert_eq!(constraint_kinds.value(row_index), constraint_kind);
        assert_eq!(
            required_boundary_sizes.value(row_index),
            required_boundary_size
        );
    }

    let (server_base_url, mut server_guard) =
        linked_builtin_spawn_wendaosearch_solver_demo_multi_route_service().await;
    let rows = fetch_plan_aware_generic_topology_filter_rows_via_manifest_discovery(
        server_base_url,
        &batch_fixture,
        constraint_kind,
        required_boundary_size,
    )
    .await?;
    server_guard.kill();

    assert_eq!(rows.len(), batch_fixture.candidates.len());
    assert!(
        rows.values().any(|row| row.accepted),
        "expected at least one accepted plan-aware worker-partition filter row, got {:?}",
        rows.keys().collect::<Vec<_>>()
    );
    for fixture in &batch_fixture.candidates {
        let row = rows.get(&fixture.candidate_id).ok_or_else(|| {
            format!(
                "missing solver_demo plan-aware worker-partition generic-topology filter row `{}`",
                fixture.candidate_id
            )
        })?;
        assert_eq!(row.candidate_id, fixture.candidate_id);
        if row.accepted {
            assert!(
                row.structural_score > 0.0,
                "unexpected accepted structural_score for `{}`: {}",
                fixture.candidate_id,
                row.structural_score
            );
            assert_eq!(
                row.pin_assignment.len(),
                expected_boundary_size,
                "unexpected accepted pin_assignment for `{}`: {:?}",
                fixture.candidate_id,
                row.pin_assignment
            );
            assert_eq!(row.rejection_reason, "");
        } else {
            assert!(
                row.pin_assignment.len() <= expected_boundary_size,
                "unexpected rejected pin_assignment for `{}`: {:?}",
                fixture.candidate_id,
                row.pin_assignment
            );
            assert!(
                !row.rejection_reason.is_empty(),
                "expected rejection reason for rejected row `{}`",
                fixture.candidate_id
            );
        }
    }

    Ok(())
}
