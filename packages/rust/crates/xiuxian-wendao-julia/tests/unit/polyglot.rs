use super::{
    memory_julia_compute_config_readiness, memory_julia_compute_manifest_row_ref,
    memory_julia_compute_profile_ref, memory_julia_compute_profile_refs,
    memory_julia_compute_readiness_evidence, memory_julia_compute_readiness_snapshot,
    memory_julia_compute_snapshot,
};
use crate::memory::{
    MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID, MemoryJuliaComputeManifestRow,
    MemoryJuliaComputeProfile,
};
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, BenchmarkState, ContractOwner, PolyglotLane, ReadinessState,
    RejectionReason, WarmupState,
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
    assert_eq!(reference.owner, ContractOwner::WendaoJulia);
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
    assert_eq!(reference.owner, ContractOwner::WendaoJulia);
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
            .all(|reference| reference.owner == ContractOwner::WendaoJulia)
    );
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
fn memory_julia_snapshot_materializes_profile_refs_and_readiness() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot =
        memory_julia_compute_snapshot(&runtime).expect("memory Julia snapshot should validate");

    assert_eq!(
        snapshot.route_refs().len(),
        MemoryJuliaComputeProfile::ALL.len()
    );
    assert!(
        snapshot
            .route_refs()
            .iter()
            .all(|reference| reference.owner == ContractOwner::WendaoJulia)
    );
    assert_eq!(
        snapshot
            .evidence_for_lane(PolyglotLane::JuliaCompute)
            .map(|evidence| evidence.readiness),
        Some(ReadinessState::Ready)
    );
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
fn readiness_snapshot_materializes_ref_budget_and_evidence() {
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
    )
    .expect("memory Julia readiness snapshot should validate");

    assert_eq!(snapshot.route_refs().len(), 1);
    assert_eq!(snapshot.admission_budgets().len(), 1);
    assert_eq!(
        snapshot
            .evidence_for_lane(PolyglotLane::JuliaCompute)
            .map(|evidence| evidence.readiness),
        Some(ReadinessState::Degraded)
    );
}
