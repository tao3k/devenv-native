use crate::wendao_contracts::{
    MemoryJuliaComputeReadinessInput, memory_julia_compute_readiness_evidence,
    memory_julia_compute_readiness_snapshot, memory_julia_compute_schedule_plan,
    memory_julia_compute_snapshot,
};
use crate::{
    AdmissionDecision, BenchmarkState, JuliaComputeTaskShape, JuliaProfileSchedulingFacts,
    JuliaRuntimeStats, JuliaScheduleAction, JuliaScheduleReason, MemoryJuliaComputeProfile,
    PolyglotLane, ReadinessState, SnapshotInvariantError, WarmupState,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};

#[test]
fn memory_julia_readiness_evidence_is_owned_by_orchestrator_contracts() {
    let config = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        max_in_flight_requests: 6,
        fallback_mode: MemoryJuliaComputeFallbackMode::Rust,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let evidence = memory_julia_compute_readiness_evidence(MemoryJuliaComputeReadinessInput {
        runtime: &config,
        profile: MemoryJuliaComputeProfile::MemoryGateScore,
        warmup: WarmupState::Ready,
        benchmark: BenchmarkState::WithinThreshold,
        active_in_flight: 2,
        queue_depth: 1,
    });

    assert_eq!(evidence.lane, PolyglotLane::JuliaCompute);
    assert_eq!(
        evidence.profile_id,
        MemoryJuliaComputeProfile::MemoryGateScore.profile_id()
    );
    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(evidence.max_in_flight, Some(6));
    assert!(evidence.fallback_available);
    assert_eq!(
        evidence.to_admission_budget().decide(),
        AdmissionDecision::Allow {
            lane: PolyglotLane::JuliaCompute,
            remaining_permits: 4,
        }
    );
}

#[test]
fn memory_julia_readiness_snapshot_materializes_profile_budget_and_lane_evidence()
-> Result<(), SnapshotInvariantError> {
    let config = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        max_in_flight_requests: 4,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot = memory_julia_compute_readiness_snapshot(MemoryJuliaComputeReadinessInput {
        runtime: &config,
        profile: MemoryJuliaComputeProfile::MemoryPlanTuning,
        warmup: WarmupState::Ready,
        benchmark: BenchmarkState::AboveThreshold,
        active_in_flight: 1,
        queue_depth: 0,
    })?;

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

#[test]
fn memory_julia_snapshot_materializes_runtime_profile_refs() -> Result<(), SnapshotInvariantError> {
    let config = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot = memory_julia_compute_snapshot(&config)?;

    assert_eq!(
        snapshot.route_refs().len(),
        MemoryJuliaComputeProfile::ALL.len()
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
fn memory_julia_schedule_plan_uses_runtime_fallback_fact() {
    let config = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        fallback_mode: MemoryJuliaComputeFallbackMode::Rust,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let facts = JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new()
            .with_warmup(WarmupState::Cold)
            .with_benchmark(BenchmarkState::WithinThreshold)
            .with_latency_ms(Some(20), Some(120)),
    )
    .with_deadline_ms(Some(50));

    let plan = memory_julia_compute_schedule_plan(
        &config,
        MemoryJuliaComputeProfile::MemoryGateScore,
        JuliaComputeTaskShape::new().with_rows(8),
        facts,
    );

    assert_eq!(plan.action, JuliaScheduleAction::Fallback);
    assert_eq!(plan.reason, JuliaScheduleReason::DeadlineTooTight);
    assert!(plan.fallback_available);
}
