use super::{
    BenchmarkState, GraphStructuralRouteKind, JuliaProfileSchedulingFacts, JuliaScheduleAction,
    JuliaScheduleReason, JuliaThreadPinningDiagnostics, JuliaThreadPinningState,
    JuliaThreadTopology, LaneCapability, LinkGraphJuliaRerankRuntimeConfig, ReadinessState,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WarmupState, gnn_host_probe_report,
    heavy_graph_shape, link_graph_full_structural_host_probe_report, page_index_host_probe_report,
    page_index_planner_action_host_probe_report,
    wendao_graph_gnn_accelerator_diagnostics_from_host_probe,
    wendao_graph_gnn_readiness_evidence_from_host_probe,
    wendao_graph_gnn_reasoning_readiness_evidence, wendao_graph_gnn_reasoning_schedule_plan,
    wendao_graph_gnn_runtime_stats_from_host_probe, wendao_graph_link_evidence_readiness_evidence,
    wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe,
    wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe,
    wendao_graph_link_evidence_schedule_plan, wendao_graph_page_index_reasoning_readiness_evidence,
    wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe,
    wendao_graph_page_index_reasoning_schedule_plan,
    wendaosearch_graph_structural_readiness_evidence,
    wendaosearch_legacy_rerank_readiness_evidence, with_julia_thread_pinning_diagnostics,
};

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
        schema_version: Some(String::new().into()),
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
