use super::{
    BenchmarkState, JuliaProfileSchedulingFacts, JuliaRuntimeStats, JuliaScheduleAction,
    JuliaTaskComplexityClass, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
    WarmupState, graph_algorithm_workload, link_graph_full_structural_host_probe_report,
    relationship_search_evidence_row, required, scheduling_facts,
    wendaograph_algorithm_schedule_plan, wendaograph_algorithm_task_shape,
    wendaograph_frontier_algorithm_ref, wendaograph_frontier_schedule_plan,
    wendaograph_frontier_task_shape,
    wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe,
    wendaograph_relationship_search_evidence_from_full_structural_host_probe,
};

#[test]
fn wendaograph_algorithm_task_shape_preserves_catalog_complexity_and_workload() {
    let shape = required(
        wendaograph_algorithm_task_shape("link_graph.topology_core", graph_algorithm_workload()),
        "link_graph.topology_core task shape",
    );

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
    let anchor = required(
        wendaograph_frontier_algorithm_ref("anchor_query"),
        "anchor_query frontier algorithm ref",
    );
    let relation = required(
        wendaograph_frontier_algorithm_ref("relation_path"),
        "relation_path frontier algorithm ref",
    );
    let page_index = required(
        wendaograph_frontier_algorithm_ref("page_index_seed"),
        "page_index_seed frontier algorithm ref",
    );
    let source = required(
        wendaograph_frontier_algorithm_ref("source_path"),
        "source_path frontier algorithm ref",
    );

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

    let relation_shape = required(
        wendaograph_frontier_task_shape("relation_path", graph_algorithm_workload()),
        "relation_path task shape",
    );
    let relation_plan = required(
        wendaograph_frontier_schedule_plan("relation_path", graph_algorithm_workload(), facts),
        "relation_path schedule plan",
    );
    let page_index_plan = required(
        wendaograph_frontier_schedule_plan("page_index_seed", graph_algorithm_workload(), facts),
        "page_index_seed schedule plan",
    );

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

    let link_graph = required(
        wendaograph_algorithm_schedule_plan(
            "link_graph.topology_core",
            graph_algorithm_workload(),
            facts,
        ),
        "link_graph.topology_core schedule plan",
    );
    let page_index = required(
        wendaograph_algorithm_schedule_plan(
            "page_index.planner_actions",
            graph_algorithm_workload(),
            facts,
        ),
        "page_index.planner_actions schedule plan",
    );
    let gnn = required(
        wendaograph_algorithm_schedule_plan("gnn.node_scores", graph_algorithm_workload(), facts),
        "gnn.node_scores schedule plan",
    );
    let strategy_flow = required(
        wendaograph_algorithm_schedule_plan(
            "search_strategy_flow.frontier_rows",
            graph_algorithm_workload(),
            facts,
        ),
        "search_strategy_flow.frontier_rows schedule plan",
    );
    let relationship_search = required(
        wendaograph_algorithm_schedule_plan(
            "relationship_search.ppr_like_relatedness",
            graph_algorithm_workload(),
            facts,
        ),
        "relationship_search.ppr_like_relatedness schedule plan",
    );

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
