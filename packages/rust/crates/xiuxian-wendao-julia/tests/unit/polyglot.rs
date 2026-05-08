use super::{
    JuliaProfileSchedulingFacts, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    WendaoGraphAlgorithmWorkload, julia_graph_compute_profile_refs, julia_graph_compute_snapshot,
    memory_julia_compute_config_readiness, memory_julia_compute_manifest_row_ref,
    memory_julia_compute_profile_ref, memory_julia_compute_profile_refs,
    memory_julia_compute_readiness_evidence, memory_julia_compute_readiness_snapshot,
    memory_julia_compute_schedule_plan, memory_julia_compute_snapshot,
    wendao_graph_gnn_accelerator_diagnostics_from_host_probe,
    wendao_graph_gnn_readiness_evidence_from_host_probe, wendao_graph_gnn_reasoning_profile_ref,
    wendao_graph_gnn_reasoning_readiness_evidence, wendao_graph_gnn_reasoning_schedule_plan,
    wendao_graph_gnn_runtime_stats_from_host_probe, wendao_graph_link_evidence_profile_ref,
    wendao_graph_link_evidence_readiness_evidence,
    wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe,
    wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe,
    wendao_graph_link_evidence_schedule_plan, wendao_graph_page_index_reasoning_profile_ref,
    wendao_graph_page_index_reasoning_readiness_evidence,
    wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe,
    wendao_graph_page_index_reasoning_schedule_plan, wendaograph_algorithm_ref,
    wendaograph_algorithm_refs, wendaograph_algorithm_refs_for_profile,
    wendaograph_algorithm_schedule_plan, wendaograph_algorithm_task_shape,
    wendaograph_frontier_algorithm_ref, wendaograph_frontier_schedule_plan,
    wendaograph_frontier_task_shape, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe,
    wendaograph_relationship_search_evidence_from_full_structural_host_probe,
    wendaograph_search_strategy_flow_algorithm_refs, wendaosearch_graph_structural_profile_ref,
    wendaosearch_graph_structural_readiness_evidence, wendaosearch_graph_structural_schedule_plan,
    wendaosearch_legacy_rerank_profile_ref, wendaosearch_legacy_rerank_readiness_evidence,
    wendaosearch_legacy_rerank_schedule_plan, with_julia_thread_pinning_diagnostics,
};
use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, LinkGraphJuliaRerankRuntimeConfig,
};
use crate::integration_support::{
    WendaoGraphGnnBackendLoadDiagnostics, WendaoGraphGnnHostProbeReport,
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
};
use crate::memory::{
    MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID, MemoryJuliaComputeManifestRow,
    MemoryJuliaComputeProfile,
};
use crate::{
    GRAPH_STRUCTURAL_FILTER_ROUTE, GRAPH_STRUCTURAL_RERANK_ROUTE, GraphStructuralRouteKind,
    JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
};
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, BenchmarkState, ContractOwner, JuliaComputeTaskShape, JuliaRuntimeStats,
    JuliaScheduleAction, JuliaScheduleReason, JuliaTaskComplexityClass,
    JuliaThreadPinningDiagnostics, JuliaThreadPinningState, JuliaThreadTopology, LaneCapability,
    PolyglotLane, ReadinessState, RejectionReason, SnapshotInvariantError, WarmupState,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};

#[test]
fn profile_ref_projects_runtime_route_and_schema() {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v2".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.routes.memory_gate_score = "/memory/custom_gate_score".to_string();

    let reference =
        memory_julia_compute_profile_ref(&runtime, MemoryJuliaComputeProfile::MemoryGateScore);

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, "/memory/custom_gate_score");
    assert_eq!(
        reference.profile.as_deref(),
        Some(MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID)
    );
    assert_eq!(reference.schema_version.as_deref(), Some("v2"));
}

#[test]
fn manifest_row_ref_preserves_julia_owner() {
    let row = MemoryJuliaComputeManifestRow {
        family: "memory".to_string(),
        capability_id: "memory_gate_score".to_string(),
        profile_id: "memory_gate_score".to_string(),
        request_schema_id: "memory.gate_score.request.v1".to_string(),
        response_schema_id: "memory.gate_score.response.v1".to_string(),
        route: "/memory/gate_score".to_string(),
        health_route: Some("/healthz".to_string()),
        schema_version: "v1".to_string(),
        timeout_secs: Some(10),
        scenario_pack: None,
        enabled: true,
    };

    let reference = memory_julia_compute_manifest_row_ref(&row);

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, "/memory/gate_score");
    assert_eq!(reference.profile.as_deref(), Some("memory_gate_score"));
    assert_eq!(reference.schema_version.as_deref(), Some("v1"));
}

#[test]
fn profile_refs_cover_staged_memory_profiles() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let references = memory_julia_compute_profile_refs(&runtime);

    assert_eq!(references.len(), MemoryJuliaComputeProfile::ALL.len());
    assert!(
        references
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
}

#[test]
fn wendao_graph_ref_projects_link_evidence_contract() {
    let reference = wendao_graph_link_evidence_profile_ref();

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE);
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    );
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION)
    );
}

#[test]
fn wendao_graph_ref_projects_page_index_reasoning_contract() {
    let reference = wendao_graph_page_index_reasoning_profile_ref();

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(
        reference.route,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT
    );
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID)
    );
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION)
    );
}

#[test]
fn wendao_graph_ref_projects_gnn_reasoning_contract() {
    let reference = wendao_graph_gnn_reasoning_profile_ref();

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT);
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAO_GRAPH_GNN_REASONING_PROFILE_ID)
    );
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION)
    );
}

#[test]
fn wendaosearch_refs_project_structural_routes() {
    let rerank =
        wendaosearch_graph_structural_profile_ref(GraphStructuralRouteKind::StructuralRerank);
    let filter =
        wendaosearch_graph_structural_profile_ref(GraphStructuralRouteKind::ConstraintFilter);

    assert_eq!(rerank.route, GRAPH_STRUCTURAL_RERANK_ROUTE);
    assert_eq!(
        rerank.profile.as_deref(),
        Some(WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID)
    );
    assert_eq!(
        rerank.schema_version.as_deref(),
        Some(JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION)
    );
    assert_eq!(filter.route, GRAPH_STRUCTURAL_FILTER_ROUTE);
    assert_eq!(
        filter.profile.as_deref(),
        Some(WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID)
    );
}

#[test]
fn wendaosearch_legacy_ref_projects_runtime_override() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        route: Some("/custom/rerank".to_string()),
        schema_version: Some("v2".to_string()),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    let reference = wendaosearch_legacy_rerank_profile_ref(&runtime);

    assert_eq!(reference.route, "/custom/rerank");
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
    );
    assert_eq!(reference.schema_version.as_deref(), Some("v2"));
}

#[test]
fn wendaosearch_legacy_ref_uses_default_route_without_runtime_override() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let reference = wendaosearch_legacy_rerank_profile_ref(&runtime);

    assert_eq!(reference.route, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE);
    assert_eq!(reference.schema_version.as_deref(), Some("v1"));
}

#[test]
fn graph_compute_refs_cover_wendaograph_and_wendaosearch_contracts() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let references = julia_graph_compute_profile_refs(&runtime);

    assert_eq!(references.len(), 6);
    assert!(
        references
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    ));
    assert!(
        references
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID))
    );
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_GNN_REASONING_PROFILE_ID)
    ));
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
    ));
    assert!(
        references
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID))
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID))
    );
}

#[test]
fn graph_compute_snapshot_materializes_contract_refs() -> Result<(), SnapshotInvariantError> {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let snapshot = julia_graph_compute_snapshot(&runtime)?;

    assert_eq!(snapshot.route_refs().len(), 6);
    assert!(snapshot.admission_budgets().is_empty());
    assert!(snapshot.lane_evidence().is_empty());
    Ok(())
}

#[test]
fn wendaograph_algorithm_catalog_covers_staged_algorithm_families() {
    let references = wendaograph_algorithm_refs();

    assert_eq!(wendaograph_link_graph_algorithm_refs().len(), 17);
    assert_eq!(wendaograph_relationship_search_algorithm_refs().len(), 10);
    assert_eq!(wendaograph_page_index_algorithm_refs().len(), 3);
    assert_eq!(wendaograph_search_strategy_flow_algorithm_refs().len(), 4);
    assert_eq!(wendaograph_gnn_algorithm_refs().len(), 4);
    assert_eq!(references.len(), 38);
    assert!(
        references
            .iter()
            .all(|reference| reference.capability == LaneCapability::GraphEvidenceCompute)
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "link_graph.topology_community_frontier")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "relationship_search.hnsw_semantic_fanout")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "page_index.planner_actions")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "search_strategy_flow.frontier_rows")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "gnn.node_scores")
    );
}

#[test]
fn wendaograph_algorithm_catalog_groups_by_profile() {
    let link_graph = wendaograph_algorithm_refs_for_profile(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
    let page_index =
        wendaograph_algorithm_refs_for_profile(WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID);
    let gnn = wendaograph_algorithm_refs_for_profile(WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);

    assert_eq!(link_graph.len(), 27);
    assert_eq!(page_index.len(), 7);
    assert_eq!(gnn.len(), 4);
    assert!(wendaograph_algorithm_refs_for_profile("missing").is_empty());
    assert!(
        page_index
            .iter()
            .all(|reference| matches!(reference.family, "page_index" | "search_strategy_flow"))
    );
}

#[test]
fn wendaograph_search_strategy_flow_catalog_aligns_graph_owned_contract() {
    let references = wendaograph_search_strategy_flow_algorithm_refs();

    let candidates = wendaograph_algorithm_ref("search_strategy_flow.candidate_rows").unwrap();
    let transitions = wendaograph_algorithm_ref("search_strategy_flow.transition_rows").unwrap();
    let frontier = wendaograph_algorithm_ref("search_strategy_flow.frontier_rows").unwrap();
    let tables = wendaograph_algorithm_ref("search_strategy_flow.tables").unwrap();

    assert_eq!(references.len(), 4);
    assert!(references.iter().all(|reference| {
        reference.family == "search_strategy_flow"
            && reference.profile_id == WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
            && reference.capability == LaneCapability::GraphEvidenceCompute
    }));
    assert_eq!(
        candidates.julia_entrypoint,
        "WendaoGraph.strategy_flow_candidate_rows"
    );
    assert_eq!(candidates.output_table, Some("strategy_candidates"));
    assert_eq!(
        transitions.julia_entrypoint,
        "WendaoGraph.strategy_flow_transition_rows"
    );
    assert_eq!(transitions.output_table, Some("strategy_transitions"));
    assert_eq!(transitions.complexity, JuliaTaskComplexityClass::Simple);
    assert_eq!(
        frontier.julia_entrypoint,
        "WendaoGraph.strategy_flow_frontier_rows"
    );
    assert_eq!(frontier.output_table, Some("strategy_frontier"));
    assert_eq!(tables.julia_entrypoint, "WendaoGraph.strategy_flow_tables");
    assert_eq!(tables.output_table, None);
}

#[test]
fn wendaograph_relationship_search_catalog_aligns_graph_search_families() {
    let references = wendaograph_relationship_search_algorithm_refs();

    let hnsw = wendaograph_algorithm_ref("relationship_search.hnsw_semantic_fanout").unwrap();
    let moc = wendaograph_algorithm_ref("relationship_search.moc_community_grouping").unwrap();
    let ppr = wendaograph_algorithm_ref("relationship_search.ppr_like_relatedness").unwrap();
    let ranking = wendaograph_algorithm_ref("relationship_search.graph_search_ranking").unwrap();
    let traversal =
        wendaograph_algorithm_ref("relationship_search.large_object_graph_traversal").unwrap();
    let snapshot =
        wendaograph_algorithm_ref("relationship_search.graph_snapshot_traversal").unwrap();

    assert_eq!(references.len(), 10);
    assert!(references.iter().all(|reference| {
        reference.family == "relationship_search"
            && reference.profile_id == WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID
    }));
    assert_eq!(hnsw.julia_entrypoint, "WendaoGraph.hnsw_neighbor_rows");
    assert_eq!(hnsw.output_table, Some("semantic_neighbors"));
    assert!(hnsw.is_heavy());
    assert_eq!(moc.julia_entrypoint, "WendaoGraph.topology_community_rows");
    assert_eq!(
        ppr.julia_entrypoint,
        "WendaoGraph.multi_plane_diffusion_scores"
    );
    assert_eq!(ranking.julia_entrypoint, "WendaoGraph.link_frontier_rows");
    assert_eq!(traversal.julia_entrypoint, "WendaoGraph.sparse_adjacency");
    assert_eq!(traversal.output_table, None);
    assert_eq!(
        snapshot.julia_entrypoint,
        "WendaoGraph.build_graph_snapshot"
    );
    assert_eq!(snapshot.output_table, None);
}

#[test]
fn wendaograph_algorithm_catalog_marks_heavy_julia_helpers() {
    let core = wendaograph_algorithm_ref("link_graph.topology_core").unwrap();
    let diffusion = wendaograph_algorithm_ref("link_graph.diffusion_scores").unwrap();
    let gnn = wendaograph_algorithm_ref("gnn.node_scores").unwrap();
    let trace = wendaograph_algorithm_ref("page_index.disclosure_trace").unwrap();
    let transition = wendaograph_algorithm_ref("search_strategy_flow.transition_rows").unwrap();

    assert!(core.is_heavy());
    assert_eq!(core.output_table, Some("topology_core"));
    assert_eq!(core.julia_entrypoint, "WendaoGraph.topology_core_rows");
    assert!(diffusion.is_heavy());
    assert_eq!(
        diffusion.julia_entrypoint,
        "WendaoGraph.multi_plane_diffusion_scores"
    );
    assert!(gnn.is_heavy());
    assert_eq!(gnn.profile_id, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);
    assert_eq!(trace.complexity, JuliaTaskComplexityClass::Simple);
    assert_eq!(transition.complexity, JuliaTaskComplexityClass::Simple);
}

#[test]
fn wendaograph_algorithm_task_shape_preserves_catalog_complexity_and_workload() {
    let shape =
        wendaograph_algorithm_task_shape("link_graph.topology_core", graph_algorithm_workload())
            .unwrap();

    assert_eq!(shape.rows, 24);
    assert_eq!(shape.nodes, 1_500);
    assert_eq!(shape.edges, 12_000);
    assert_eq!(shape.feature_columns, 18);
    assert_eq!(shape.byte_size, 2 * 1024 * 1024);
    assert_eq!(shape.complexity, JuliaTaskComplexityClass::Heavy);
    assert_eq!(
        shape.batchability_key.as_deref(),
        Some("wendaograph:wendao_graph_link_evidence:link_graph.topology_core")
    );
    assert!(wendaograph_algorithm_task_shape("missing", graph_algorithm_workload()).is_none());
}

#[test]
fn wendaograph_frontier_mapping_routes_evidence_kinds_to_graph_algorithms() {
    let anchor = wendaograph_frontier_algorithm_ref("anchor_query").unwrap();
    let relation = wendaograph_frontier_algorithm_ref("relation_path").unwrap();
    let page_index = wendaograph_frontier_algorithm_ref("page_index_seed").unwrap();
    let source = wendaograph_frontier_algorithm_ref("source_path").unwrap();

    assert_eq!(
        anchor.algorithm_id,
        "relationship_search.hnsw_semantic_fanout"
    );
    assert_eq!(
        relation.algorithm_id,
        "relationship_search.ppr_like_relatedness"
    );
    assert_eq!(page_index.algorithm_id, "page_index.reasoning_frontier");
    assert_eq!(
        source.algorithm_id,
        "relationship_search.graph_search_ranking"
    );
    assert_eq!(
        page_index.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert_eq!(source.profile_id, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
    assert!(wendaograph_frontier_algorithm_ref("authority_order").is_none());
    assert!(wendaograph_frontier_algorithm_ref("negative_guard").is_none());
}

#[test]
fn wendaograph_frontier_schedule_plan_dispatches_warm_batchable_work() {
    let facts = scheduling_facts(WarmupState::Ready, BenchmarkState::WithinThreshold)
        .with_max_in_flight(Some(4))
        .with_target_latency_ms(Some(250));

    let relation_shape =
        wendaograph_frontier_task_shape("relation_path", graph_algorithm_workload()).unwrap();
    let relation_plan =
        wendaograph_frontier_schedule_plan("relation_path", graph_algorithm_workload(), facts)
            .unwrap();
    let page_index_plan =
        wendaograph_frontier_schedule_plan("page_index_seed", graph_algorithm_workload(), facts)
            .unwrap();

    assert_eq!(relation_shape.complexity, JuliaTaskComplexityClass::Heavy);
    assert_eq!(relation_plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(
        relation_plan.profile_id,
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID
    );
    assert_eq!(page_index_plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(
        page_index_plan.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert!(
        wendaograph_frontier_schedule_plan("negative_guard", graph_algorithm_workload(), facts)
            .is_none()
    );
}

#[test]
fn wendaograph_algorithm_schedule_plan_routes_by_algorithm_profile() {
    let facts = scheduling_facts(WarmupState::Ready, BenchmarkState::WithinThreshold)
        .with_max_in_flight(Some(4))
        .with_target_latency_ms(Some(250));

    let link_graph = wendaograph_algorithm_schedule_plan(
        "link_graph.topology_core",
        graph_algorithm_workload(),
        facts,
    )
    .unwrap();
    let page_index = wendaograph_algorithm_schedule_plan(
        "page_index.planner_actions",
        graph_algorithm_workload(),
        facts,
    )
    .unwrap();
    let gnn =
        wendaograph_algorithm_schedule_plan("gnn.node_scores", graph_algorithm_workload(), facts)
            .unwrap();
    let strategy_flow = wendaograph_algorithm_schedule_plan(
        "search_strategy_flow.frontier_rows",
        graph_algorithm_workload(),
        facts,
    )
    .unwrap();
    let relationship_search = wendaograph_algorithm_schedule_plan(
        "relationship_search.ppr_like_relatedness",
        graph_algorithm_workload(),
        facts,
    )
    .unwrap();

    assert_eq!(link_graph.action, JuliaScheduleAction::Dispatch);
    assert_eq!(link_graph.profile_id, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
    assert_eq!(relationship_search.action, JuliaScheduleAction::Dispatch);
    assert_eq!(
        relationship_search.profile_id,
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID
    );
    assert_eq!(page_index.action, JuliaScheduleAction::Dispatch);
    assert_eq!(
        page_index.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert_eq!(strategy_flow.action, JuliaScheduleAction::Dispatch);
    assert_eq!(
        strategy_flow.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert_eq!(gnn.action, JuliaScheduleAction::Dispatch);
    assert_eq!(gnn.profile_id, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);
    assert!(
        wendaograph_algorithm_schedule_plan("missing", graph_algorithm_workload(), facts).is_none()
    );
}

#[test]
fn wendaograph_relationship_search_evidence_projects_full_structural_host_probe() {
    let report = link_graph_full_structural_host_probe_report();
    let facts = JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new().with_benchmark(BenchmarkState::WithinThreshold),
    )
    .with_max_in_flight(Some(4))
    .with_target_latency_ms(Some(10));

    let evidence = wendaograph_relationship_search_evidence_from_full_structural_host_probe(
        &report,
        graph_algorithm_workload(),
        facts,
    );

    assert_eq!(evidence.len(), 10);
    assert!(
        evidence
            .iter()
            .all(|row| row.runtime_stats.p50_latency_ms == Some(1)
                && row.runtime_stats.p95_latency_ms == Some(1)
                && row.schedule_plan.action == JuliaScheduleAction::Dispatch
                && row.schedule_plan.profile_id == WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    );

    let hnsw =
        relationship_search_evidence_row(&evidence, "relationship_search.hnsw_semantic_fanout");
    let moc =
        relationship_search_evidence_row(&evidence, "relationship_search.moc_community_grouping");
    let ppr =
        relationship_search_evidence_row(&evidence, "relationship_search.ppr_like_relatedness");
    let ranking =
        relationship_search_evidence_row(&evidence, "relationship_search.graph_search_ranking");
    let traversal = relationship_search_evidence_row(
        &evidence,
        "relationship_search.large_object_graph_traversal",
    );

    assert_eq!(hnsw.probe_table, Some("semantic_overlay"));
    assert_eq!(hnsw.probe_rows, Some(2));
    assert_eq!(moc.probe_table, Some("topology_communities"));
    assert_eq!(moc.probe_rows, Some(4));
    assert_eq!(ppr.probe_table, Some("diffusion_scores"));
    assert_eq!(ppr.probe_rows, Some(4));
    assert_eq!(ranking.probe_table, Some("link_frontier"));
    assert_eq!(ranking.probe_rows, Some(3));
    assert_eq!(traversal.probe_table, Some("components"));
    assert_eq!(traversal.probe_rows, Some(4));
}

#[test]
fn wendaograph_relationship_search_evidence_ignores_unknown_or_non_relationship_ids() {
    let report = link_graph_full_structural_host_probe_report();
    let facts = JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new().with_benchmark(BenchmarkState::WithinThreshold),
    );

    assert!(
        wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
            "missing",
            &report,
            graph_algorithm_workload(),
            facts,
        )
        .is_none()
    );
    assert!(
        wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
            "link_graph.diffusion_scores",
            &report,
            graph_algorithm_workload(),
            facts,
        )
        .is_none()
    );
}

#[test]
fn graph_readiness_evidence_projects_contract_capabilities() {
    let graph_evidence = wendao_graph_link_evidence_readiness_evidence(
        WarmupState::Ready,
        BenchmarkState::NotRequired,
        Some(2),
        1,
        0,
    );
    let graph_search = wendaosearch_graph_structural_readiness_evidence(
        GraphStructuralRouteKind::StructuralRerank,
        WarmupState::Ready,
        BenchmarkState::WithinThreshold,
        Some(3),
        1,
        1,
    );
    let page_index = wendao_graph_page_index_reasoning_readiness_evidence(
        WarmupState::Ready,
        BenchmarkState::NotRequired,
        Some(2),
        0,
        0,
    );
    let gnn = wendao_graph_gnn_reasoning_readiness_evidence(
        WarmupState::Ready,
        BenchmarkState::NotRequired,
        Some(2),
        0,
        0,
    );

    assert_eq!(
        graph_evidence.capability,
        LaneCapability::GraphEvidenceCompute
    );
    assert_eq!(page_index.capability, LaneCapability::GraphEvidenceCompute);
    assert_eq!(gnn.capability, LaneCapability::GraphEvidenceCompute);
    assert_eq!(graph_search.capability, LaneCapability::GraphSearchCompute);
    assert_eq!(graph_evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(page_index.readiness_state(), ReadinessState::Ready);
    assert_eq!(gnn.readiness_state(), ReadinessState::Ready);
    assert_eq!(graph_search.readiness_state(), ReadinessState::Ready);
}

#[test]
fn graph_readiness_preserves_thread_pinning_diagnostics_without_gating_readiness() {
    let diagnostics = JuliaThreadPinningDiagnostics::new(
        JuliaThreadPinningState::Applied,
        JuliaThreadTopology::new(8, 12).with_physical_core_count(Some(6)),
    )
    .with_requested_policy("cores")
    .with_applied_policy("cores")
    .with_pinned_thread_count(Some(8))
    .with_platform("Darwin-arm64");
    let evidence = with_julia_thread_pinning_diagnostics(
        wendaosearch_graph_structural_readiness_evidence(
            GraphStructuralRouteKind::StructuralRerank,
            WarmupState::Ready,
            BenchmarkState::WithinThreshold,
            Some(8),
            1,
            0,
        ),
        diagnostics,
    );

    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(
        evidence
            .thread_pinning_diagnostics
            .as_ref()
            .map(|diagnostics| diagnostics.state),
        Some(JuliaThreadPinningState::Applied)
    );
}

#[test]
fn gnn_host_probe_projects_runtime_stats_and_accelerator_diagnostics() {
    let report = gnn_host_probe_report();
    let stats = wendao_graph_gnn_runtime_stats_from_host_probe(&report);
    let diagnostics = wendao_graph_gnn_accelerator_diagnostics_from_host_probe(&report);
    let evidence = wendao_graph_gnn_readiness_evidence_from_host_probe(&report, Some(2), 1, 0);

    assert_eq!(stats.warmup, WarmupState::Ready);
    assert_eq!(stats.benchmark, BenchmarkState::NotRequired);
    assert_eq!(stats.p50_latency_ms, Some(19));
    assert_eq!(stats.p95_latency_ms, Some(389));
    assert_eq!(diagnostics.len(), 3);
    assert_eq!(diagnostics[0].backend, "metal");
    assert!(diagnostics[0].loaded);
    assert!(diagnostics[0].functional);
    assert_eq!(diagnostics[0].observed_output_count, Some(4));
    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(
        evidence
            .accelerator_diagnostics
            .iter()
            .find(|diagnostics| diagnostics.backend == "metal")
            .and_then(|diagnostics| diagnostics.observed_output_count),
        Some(4)
    );
}

#[test]
fn gnn_schedule_dispatches_warm_heavy_shape_from_probe_stats() {
    let report = gnn_host_probe_report();
    let facts = JuliaProfileSchedulingFacts::new(
        wendao_graph_gnn_runtime_stats_from_host_probe(&report)
            .with_benchmark(BenchmarkState::WithinThreshold),
    )
    .with_max_in_flight(Some(4))
    .with_target_latency_ms(Some(500));

    let plan = wendao_graph_gnn_reasoning_schedule_plan(heavy_graph_shape(), facts);

    assert_eq!(plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaAdvantage);
    assert_eq!(plan.profile_id, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);
}

#[test]
fn link_graph_host_probe_projects_runtime_stats_and_readiness() {
    let report = link_graph_full_structural_host_probe_report();
    let stats = wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe(&report);
    let evidence = wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe(
        &report,
        Some(4),
        1,
        0,
    );

    assert_eq!(stats.warmup, WarmupState::Ready);
    assert_eq!(stats.benchmark, BenchmarkState::NotRequired);
    assert_eq!(stats.p50_latency_ms, Some(1));
    assert_eq!(stats.p95_latency_ms, Some(1));
    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(evidence.profile_id, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
    assert_eq!(evidence.max_in_flight, Some(4));
    assert_eq!(report.topology_community_summary_rows, 2);
    assert_eq!(report.base.frontier_rows, 3);
}

#[test]
fn link_graph_schedule_dispatches_from_full_structural_probe_stats() {
    let report = link_graph_full_structural_host_probe_report();
    let facts = JuliaProfileSchedulingFacts::new(
        wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe(&report)
            .with_benchmark(BenchmarkState::WithinThreshold),
    )
    .with_max_in_flight(Some(4))
    .with_target_latency_ms(Some(10));

    let plan = wendao_graph_link_evidence_schedule_plan(heavy_graph_shape(), facts);

    assert_eq!(plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaAdvantage);
    assert_eq!(plan.profile_id, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
}

#[test]
fn page_index_host_probe_projects_runtime_stats_and_readiness() {
    let report = page_index_host_probe_report();
    let planner_report = page_index_planner_action_host_probe_report();
    let stats = wendao_graph_page_index_reasoning_runtime_stats_from_host_probe(&report);
    let planner_stats =
        wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe(
            &planner_report,
        );
    let evidence = wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe(
        &report,
        Some(4),
        1,
        0,
    );

    assert_eq!(stats.warmup, WarmupState::Ready);
    assert_eq!(stats.benchmark, BenchmarkState::NotRequired);
    assert_eq!(stats.p50_latency_ms, Some(1));
    assert_eq!(stats.p95_latency_ms, Some(1));
    assert_eq!(planner_stats.p50_latency_ms, Some(1));
    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(
        evidence.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
    assert_eq!(planner_report.planner_action_rows, 3);
}

#[test]
fn page_index_schedule_dispatches_from_host_probe_stats() {
    let report = page_index_host_probe_report();
    let facts = JuliaProfileSchedulingFacts::new(
        wendao_graph_page_index_reasoning_runtime_stats_from_host_probe(&report)
            .with_benchmark(BenchmarkState::WithinThreshold),
    )
    .with_max_in_flight(Some(4))
    .with_target_latency_ms(Some(10));

    let plan = wendao_graph_page_index_reasoning_schedule_plan(heavy_graph_shape(), facts);

    assert_eq!(plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaAdvantage);
    assert_eq!(
        plan.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
}

#[test]
fn wendaosearch_legacy_readiness_invalidates_empty_schema_override() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        schema_version: Some(String::new()),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    let evidence = wendaosearch_legacy_rerank_readiness_evidence(
        &runtime,
        WarmupState::Ready,
        BenchmarkState::NotRequired,
        0,
        0,
    );

    assert_eq!(evidence.readiness_state(), ReadinessState::Disabled);
}

#[test]
fn graph_structural_schedule_dispatches_warm_heavy_shape() {
    let plan = wendaosearch_graph_structural_schedule_plan(
        GraphStructuralRouteKind::StructuralRerank,
        heavy_graph_shape(),
        scheduling_facts(WarmupState::Ready, BenchmarkState::WithinThreshold)
            .with_max_in_flight(Some(8))
            .with_target_latency_ms(Some(250)),
    );

    assert_eq!(plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaAdvantage);
    assert_eq!(plan.capability, LaneCapability::GraphSearchCompute);
    assert_eq!(plan.profile_id, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID);
    assert!(plan.selected_batch_size > 1);
}

#[test]
fn graph_link_schedule_queues_under_pressure_without_forcing_fallback() {
    let facts = scheduling_facts(WarmupState::Ready, BenchmarkState::WithinThreshold)
        .with_max_in_flight(Some(4));
    let facts = JuliaProfileSchedulingFacts {
        runtime_stats: facts.runtime_stats.with_queue(4, 2),
        ..facts
    };

    let plan = wendao_graph_link_evidence_schedule_plan(heavy_graph_shape(), facts);

    assert_eq!(plan.action, JuliaScheduleAction::Queue);
    assert_eq!(plan.reason, JuliaScheduleReason::QueuePressure);
    assert!(!plan.fallback_available);
}

#[test]
fn page_index_reasoning_schedule_dispatches_warm_heavy_shape() {
    let plan = wendao_graph_page_index_reasoning_schedule_plan(
        heavy_graph_shape(),
        scheduling_facts(WarmupState::Ready, BenchmarkState::WithinThreshold)
            .with_max_in_flight(Some(4))
            .with_target_latency_ms(Some(250)),
    );

    assert_eq!(plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaAdvantage);
    assert_eq!(plan.capability, LaneCapability::GraphEvidenceCompute);
    assert_eq!(
        plan.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
}

#[test]
fn memory_schedule_uses_runtime_rust_fallback_for_tight_cold_deadline() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        fallback_mode: MemoryJuliaComputeFallbackMode::Rust,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let facts = scheduling_facts(WarmupState::Cold, BenchmarkState::WithinThreshold)
        .with_deadline_ms(Some(50));
    let facts = JuliaProfileSchedulingFacts {
        runtime_stats: facts.runtime_stats.with_latency_ms(Some(20), Some(120)),
        ..facts
    };

    let plan = memory_julia_compute_schedule_plan(
        &runtime,
        MemoryJuliaComputeProfile::MemoryGateScore,
        memory_shape(),
        facts,
    );

    assert_eq!(plan.action, JuliaScheduleAction::Fallback);
    assert_eq!(plan.reason, JuliaScheduleReason::DeadlineTooTight);
    assert!(plan.fallback_available);
}

#[test]
fn legacy_rerank_schedule_rejects_invalid_schema_without_fallback() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        schema_version: Some(String::new()),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    let plan = wendaosearch_legacy_rerank_schedule_plan(
        &runtime,
        heavy_graph_shape(),
        scheduling_facts(WarmupState::Ready, BenchmarkState::WithinThreshold),
    );

    assert_eq!(plan.action, JuliaScheduleAction::Reject);
    assert_eq!(plan.reason, JuliaScheduleReason::ContractInvalid);
}

#[test]
fn config_readiness_maps_enabled_flag() {
    let disabled = MemoryJuliaComputeRuntimeConfig::default();
    let enabled = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    assert_eq!(
        memory_julia_compute_config_readiness(&disabled),
        ReadinessState::Disabled
    );
    assert_eq!(
        memory_julia_compute_config_readiness(&enabled),
        ReadinessState::Ready
    );
}

#[test]
fn memory_julia_snapshot_materializes_profile_refs_and_readiness()
-> Result<(), SnapshotInvariantError> {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot = memory_julia_compute_snapshot(&runtime)?;

    assert_eq!(
        snapshot.route_refs().len(),
        MemoryJuliaComputeProfile::ALL.len()
    );
    assert!(
        snapshot
            .route_refs()
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
    assert_eq!(
        snapshot
            .evidence_for_lane(PolyglotLane::JuliaCompute)
            .map(|evidence| evidence.readiness),
        Some(ReadinessState::Ready)
    );
    Ok(())
}

#[test]
fn readiness_evidence_projects_enabled_profile_facts() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        max_in_flight_requests: 4,
        fallback_mode: MemoryJuliaComputeFallbackMode::Rust,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let evidence = memory_julia_compute_readiness_evidence(
        &runtime,
        MemoryJuliaComputeProfile::EpisodicRecall,
        WarmupState::Ready,
        BenchmarkState::WithinThreshold,
        1,
        0,
    );

    assert_eq!(evidence.lane, PolyglotLane::JuliaCompute);
    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(
        evidence.to_admission_budget().decide(),
        AdmissionDecision::Allow {
            lane: PolyglotLane::JuliaCompute,
            remaining_permits: 3,
        }
    );
}

#[test]
fn readiness_evidence_saturates_wide_admission_window_values() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        max_in_flight_requests: u64::MAX,
        fallback_mode: MemoryJuliaComputeFallbackMode::Rust,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let evidence = memory_julia_compute_readiness_evidence(
        &runtime,
        MemoryJuliaComputeProfile::EpisodicRecall,
        WarmupState::Ready,
        BenchmarkState::WithinThreshold,
        0,
        0,
    );

    assert_eq!(evidence.to_admission_budget().max_in_flight, Some(u32::MAX));
}

#[test]
fn readiness_evidence_disables_disabled_runtime() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: false,
        schema_version: "v1".to_string(),
        max_in_flight_requests: 4,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let evidence = memory_julia_compute_readiness_evidence(
        &runtime,
        MemoryJuliaComputeProfile::MemoryCalibration,
        WarmupState::Ready,
        BenchmarkState::WithinThreshold,
        0,
        0,
    );

    assert_eq!(evidence.readiness_state(), ReadinessState::Disabled);
    assert_eq!(
        evidence.to_admission_budget().decide(),
        AdmissionDecision::Reject {
            lane: PolyglotLane::JuliaCompute,
            reason: RejectionReason::LaneDisabled,
        }
    );
}

#[test]
fn readiness_snapshot_materializes_ref_budget_and_evidence() -> Result<(), SnapshotInvariantError> {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        max_in_flight_requests: 4,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot = memory_julia_compute_readiness_snapshot(
        &runtime,
        MemoryJuliaComputeProfile::MemoryPlanTuning,
        WarmupState::Ready,
        BenchmarkState::AboveThreshold,
        1,
        0,
    )?;

    assert_eq!(snapshot.route_refs().len(), 1);
    assert_eq!(snapshot.admission_budgets().len(), 1);
    assert_eq!(
        snapshot
            .evidence_for_lane(PolyglotLane::JuliaCompute)
            .map(|evidence| evidence.readiness),
        Some(ReadinessState::Degraded)
    );
    Ok(())
}

fn scheduling_facts(warmup: WarmupState, benchmark: BenchmarkState) -> JuliaProfileSchedulingFacts {
    JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new()
            .with_warmup(warmup)
            .with_benchmark(benchmark)
            .with_latency_ms(Some(30), Some(90)),
    )
}

fn heavy_graph_shape() -> JuliaComputeTaskShape {
    JuliaComputeTaskShape::new()
        .with_rows(24)
        .with_graph_size(1_500, 12_000)
        .with_feature_columns(18)
        .with_byte_size(2 * 1024 * 1024)
        .with_batchability_key("graph:v1")
        .with_complexity(JuliaTaskComplexityClass::Heavy)
}

fn memory_shape() -> JuliaComputeTaskShape {
    JuliaComputeTaskShape::new()
        .with_rows(1)
        .with_feature_columns(6)
        .with_byte_size(64 * 1024)
        .with_complexity(JuliaTaskComplexityClass::Simple)
}

fn graph_algorithm_workload() -> WendaoGraphAlgorithmWorkload {
    WendaoGraphAlgorithmWorkload::new()
        .with_rows(24)
        .with_graph_size(1_500, 12_000)
        .with_feature_columns(18)
        .with_byte_size(2 * 1024 * 1024)
}

fn relationship_search_evidence_row<'a>(
    evidence: &'a [super::WendaoGraphRelationshipSearchEvidence],
    algorithm_id: &str,
) -> &'a super::WendaoGraphRelationshipSearchEvidence {
    evidence
        .iter()
        .find(|row| row.algorithm.algorithm_id == algorithm_id)
        .unwrap()
}

fn gnn_host_probe_report() -> WendaoGraphGnnHostProbeReport {
    WendaoGraphGnnHostProbeReport {
        sample_count: 2,
        first_ms: 26_186.319,
        warm_min_ms: 18.735,
        warm_median_ms: 18.735,
        warm_p95_ms: 388.719,
        warm_max_ms: 388.719,
        node_count: 4,
        edge_count: 4,
        feature_rows: 7,
        feature_cols: 4,
        score_count: 4,
        frontier_rows: 3,
        backend_load: WendaoGraphGnnBackendLoadDiagnostics {
            metal_loaded: true,
            cuda_loaded: false,
            amdgpu_loaded: false,
        },
        metal_functional: true,
        metal_score_count: 4,
    }
}

fn link_graph_full_structural_host_probe_report()
-> WendaoGraphLinkGraphFullStructuralHostProbeReport {
    WendaoGraphLinkGraphFullStructuralHostProbeReport {
        base: WendaoGraphLinkGraphHostProbeReport {
            mode: "semantic-neighbors".to_owned(),
            node_count: 4,
            edge_count: 2,
            semantic_neighbor_count: 1,
            sample_count: 3,
            first_ms: 9_485.249,
            warm_min_ms: 0.555,
            warm_median_ms: 0.555,
            warm_p95_ms: 0.742,
            warm_max_ms: 0.742,
            graph_metric_rows: 4,
            topology_candidate_rows: 1,
            semantic_overlay_rows: 2,
            diffusion_rows: 4,
            frontier_rows: 3,
        },
        component_rows: 4,
        topology_profile_rows: 4,
        topology_bottleneck_rows: 4,
        topology_community_rows: 4,
        topology_cover_rows: 4,
        topology_core_rows: 4,
        topology_boundary_rows: 4,
        topology_transition_rows: 2,
        topology_gateway_rows: 4,
        topology_community_summary_rows: 2,
        topology_community_link_rows: 0,
        topology_community_frontier_rows: 1,
    }
}

fn page_index_host_probe_report() -> WendaoGraphPageIndexHostProbeReport {
    WendaoGraphPageIndexHostProbeReport {
        sample_count: 3,
        first_ms: 1_776.453,
        warm_min_ms: 0.022,
        warm_median_ms: 0.022,
        warm_p95_ms: 0.129,
        warm_max_ms: 0.129,
        frontier_rows: 3,
        trace_rows: 3,
    }
}

fn page_index_planner_action_host_probe_report() -> WendaoGraphPageIndexPlannerActionHostProbeReport
{
    WendaoGraphPageIndexPlannerActionHostProbeReport {
        base: page_index_host_probe_report(),
        planner_action_rows: 3,
        planner_expand_actions: 1,
        planner_compare_actions: 0,
        planner_jump_actions: 1,
        planner_stop_actions: 1,
    }
}
