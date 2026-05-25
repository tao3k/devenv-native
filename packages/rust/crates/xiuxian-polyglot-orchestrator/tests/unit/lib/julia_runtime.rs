use crate::{
    BenchmarkState, ContractValidationState, JuliaProfileSchedulingFacts, JuliaReadinessEvidence,
    JuliaRuntimeStats, JuliaSchedulingInput, JuliaTaskComplexityClass, LaneCapability,
    ManifestReadinessState, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID, WarmupState, WendaoGraphAlgorithmId,
    WendaoGraphAlgorithmWorkload, WendaoGraphProfileId, WendaoGraphRelationshipSearchEvidence,
    wendao_julia_runtime_profile_ids, wendaograph_algorithm_ref, wendaograph_algorithm_refs,
    wendaograph_algorithm_schedule_plan, wendaograph_algorithm_task_shape,
    wendaograph_frontier_algorithm_ref, wendaograph_frontier_schedule_plan,
    wendaograph_link_graph_algorithm_refs, wendaograph_relationship_search_algorithm_refs,
};

#[test]
fn orchestrator_projects_julia_runtime_wendao_profile_ids() {
    assert_eq!(
        wendao_julia_runtime_profile_ids(),
        [
            WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
            WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
            WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
            WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
            WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
            WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
        ]
    );
}

#[test]
fn orchestrator_exposes_julia_runtime_identity_facts() {
    assert_eq!(
        WendaoGraphAlgorithmId("relationship_search.graph_search_ranking").0,
        "relationship_search.graph_search_ranking"
    );
    assert_eq!(
        WendaoGraphProfileId(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID).0,
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID
    );

    let workload = WendaoGraphAlgorithmWorkload::new()
        .with_rows(8)
        .with_graph_size(13, 21)
        .with_feature_columns(5);
    assert_eq!(workload.rows, 8);
    assert_eq!(workload.nodes, 13);
    assert_eq!(workload.edges, 21);
    assert_eq!(workload.feature_columns, 5);
}

#[test]
fn orchestrator_projects_julia_runtime_catalog_for_scheduling() {
    let references = wendaograph_algorithm_refs();

    assert_eq!(wendaograph_link_graph_algorithm_refs().len(), 17);
    assert_eq!(wendaograph_relationship_search_algorithm_refs().len(), 10);
    assert_eq!(references.len(), 38);
    assert!(
        references
            .iter()
            .all(|reference| reference.capability == LaneCapability::GraphEvidenceCompute)
    );

    let topology = wendaograph_algorithm_ref(WendaoGraphAlgorithmId("link_graph.topology_core"))
        .unwrap_or_else(|| panic!("missing link_graph.topology_core"));
    assert_eq!(topology.complexity, JuliaTaskComplexityClass::Heavy);
    assert!(topology.is_heavy());

    let shape = wendaograph_algorithm_task_shape(
        WendaoGraphAlgorithmId("link_graph.topology_core"),
        WendaoGraphAlgorithmWorkload::new()
            .with_rows(24)
            .with_graph_size(1_500, 12_000)
            .with_feature_columns(18)
            .with_byte_size(2 * 1024 * 1024),
    )
    .unwrap_or_else(|| panic!("missing task shape"));
    assert_eq!(shape.rows, 24);
    assert_eq!(shape.complexity, JuliaTaskComplexityClass::Heavy);
    assert_eq!(
        shape.batchability_key.as_deref(),
        Some("wendaograph:wendao_graph_link_evidence:link_graph.topology_core")
    );
}

#[test]
fn orchestrator_projects_frontier_mapping_from_runtime_catalog() {
    let page_index = wendaograph_frontier_algorithm_ref("page_index_seed")
        .unwrap_or_else(|| panic!("missing page_index_seed frontier algorithm"));

    assert_eq!(page_index.algorithm_id, "page_index.reasoning_frontier");
    assert_eq!(
        page_index.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert!(wendaograph_frontier_algorithm_ref("negative_guard").is_none());
}

#[test]
fn orchestrator_plans_wendaograph_schedule_from_runtime_catalog() {
    let facts = JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new()
            .with_warmup(WarmupState::Ready)
            .with_benchmark(BenchmarkState::WithinThreshold)
            .with_latency_ms(Some(4), Some(9)),
    )
    .with_max_in_flight(Some(4));
    let workload = WendaoGraphAlgorithmWorkload::new()
        .with_rows(9)
        .with_graph_size(30, 50)
        .with_feature_columns(8);
    let plan = wendaograph_frontier_schedule_plan("page_index_seed", workload, facts)
        .unwrap_or_else(|| panic!("missing page_index_seed schedule plan"));

    assert_eq!(plan.capability, LaneCapability::GraphEvidenceCompute);
    assert_eq!(
        plan.profile_id.as_str(),
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert!(wendaograph_frontier_schedule_plan("negative_guard", workload, facts).is_none());
    assert!(
        wendaograph_algorithm_schedule_plan(
            WendaoGraphAlgorithmId("missing.algorithm"),
            workload,
            facts,
        )
        .is_none()
    );
}

#[test]
fn orchestrator_owns_julia_profile_scheduling_evidence_types() {
    let stats = JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_latency_ms(Some(12), Some(38));
    let facts = JuliaProfileSchedulingFacts::new(stats)
        .with_max_in_flight(Some(4))
        .with_fallback_available(true)
        .with_deadline_ms(Some(250))
        .with_target_latency_ms(Some(100));

    assert_eq!(facts.max_in_flight, Some(4));
    assert!(facts.fallback_available);
    assert_eq!(facts.deadline_ms, Some(250));
    assert_eq!(facts.target_latency_ms, Some(100));

    let algorithm = wendaograph_algorithm_ref(WendaoGraphAlgorithmId(
        "relationship_search.graph_search_ranking",
    ))
    .unwrap_or_else(|| panic!("missing relationship_search.graph_search_ranking"));
    let shape = algorithm.task_shape(WendaoGraphAlgorithmWorkload::new().with_rows(7));
    let readiness = JuliaReadinessEvidence::graph_evidence_profile(algorithm.profile_id)
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold);
    let schedule_plan = JuliaSchedulingInput::new(readiness, shape, stats).plan();
    let evidence = WendaoGraphRelationshipSearchEvidence {
        algorithm,
        probe_table: Some("link_frontier"),
        probe_rows: Some(11),
        runtime_stats: stats,
        schedule_plan,
    };

    assert_eq!(evidence.algorithm.algorithm_id, algorithm.algorithm_id);
    assert_eq!(evidence.probe_table, Some("link_frontier"));
    assert_eq!(evidence.probe_rows, Some(11));
}
