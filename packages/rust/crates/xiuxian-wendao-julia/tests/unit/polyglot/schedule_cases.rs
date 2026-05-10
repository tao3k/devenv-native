use super::{
    BenchmarkState, GraphStructuralRouteKind, JuliaProfileSchedulingFacts, JuliaScheduleAction,
    JuliaScheduleReason, LaneCapability, LinkGraphJuliaRerankRuntimeConfig,
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeProfile, MemoryJuliaComputeRuntimeConfig,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    WarmupState, heavy_graph_shape, memory_julia_compute_schedule_plan, memory_shape,
    scheduling_facts, wendao_graph_link_evidence_schedule_plan,
    wendao_graph_page_index_reasoning_schedule_plan, wendaosearch_graph_structural_schedule_plan,
    wendaosearch_legacy_rerank_schedule_plan,
};

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
        schema_version: Some(String::new().into()),
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
