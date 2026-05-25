use super::{
    AdmissionDecision, BenchmarkState, ContractOwner, MemoryJuliaComputeFallbackMode,
    MemoryJuliaComputeProfile, MemoryJuliaComputeRuntimeConfig, PolyglotLane, ReadinessState,
    RejectionReason, SnapshotInvariantError, WarmupState, memory_julia_compute_config_readiness,
    memory_julia_compute_readiness_evidence, memory_julia_compute_readiness_snapshot,
    memory_julia_compute_snapshot,
};

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
