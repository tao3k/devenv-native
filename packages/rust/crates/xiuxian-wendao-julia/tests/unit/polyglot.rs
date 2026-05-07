use super::{
    JuliaProfileSchedulingFacts, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID, julia_graph_compute_profile_refs,
    julia_graph_compute_snapshot, memory_julia_compute_config_readiness,
    memory_julia_compute_manifest_row_ref, memory_julia_compute_profile_ref,
    memory_julia_compute_profile_refs, memory_julia_compute_readiness_evidence,
    memory_julia_compute_readiness_snapshot, memory_julia_compute_schedule_plan,
    memory_julia_compute_snapshot, wendao_graph_link_evidence_profile_ref,
    wendao_graph_link_evidence_readiness_evidence, wendao_graph_link_evidence_schedule_plan,
    wendaosearch_graph_structural_profile_ref, wendaosearch_graph_structural_readiness_evidence,
    wendaosearch_graph_structural_schedule_plan, wendaosearch_legacy_rerank_profile_ref,
    wendaosearch_legacy_rerank_readiness_evidence, wendaosearch_legacy_rerank_schedule_plan,
};
use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, LinkGraphJuliaRerankRuntimeConfig,
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
    JuliaScheduleAction, JuliaScheduleReason, JuliaTaskComplexityClass, LaneCapability,
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

    assert_eq!(references.len(), 4);
    assert!(
        references
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
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

    assert_eq!(snapshot.route_refs().len(), 4);
    assert!(snapshot.admission_budgets().is_empty());
    assert!(snapshot.lane_evidence().is_empty());
    Ok(())
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

    assert_eq!(
        graph_evidence.capability,
        LaneCapability::GraphEvidenceCompute
    );
    assert_eq!(graph_search.capability, LaneCapability::GraphSearchCompute);
    assert_eq!(graph_evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(graph_search.readiness_state(), ReadinessState::Ready);
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
