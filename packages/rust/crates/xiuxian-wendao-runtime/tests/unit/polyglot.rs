use super::{
    document_extract_pressure_evidence, document_extract_pressure_snapshot,
    document_extract_route_ref, document_extract_schedule_plan,
    memory_julia_compute_admission_budget, runtime_polyglot_snapshot,
};
use crate::config::MemoryJuliaComputeRuntimeConfig;
use crate::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, ContractOwner, DoclingScheduleAction, DoclingScheduleReason, PolyglotLane,
    PressureLevel, ReadinessState, RejectionReason, SnapshotInvariantError,
};

#[test]
fn document_extract_ref_preserves_analyzer_route() {
    let reference = document_extract_route_ref();

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::Analyzer);
    assert_eq!(reference.route, ANALYSIS_DOCUMENT_EXTRACT_ROUTE);
    assert!(reference.profile.is_none());
    assert!(reference.schema_version.is_none());
}

#[test]
fn julia_config_projects_to_admission_budget() {
    let config = MemoryJuliaComputeRuntimeConfig {
        max_in_flight_requests: 7,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let budget = memory_julia_compute_admission_budget(
        &config,
        2,
        3,
        ReadinessState::Ready,
        PressureLevel::Medium,
    );

    assert_eq!(budget.lane, PolyglotLane::JuliaCompute);
    assert_eq!(budget.max_in_flight, Some(7));
    assert_eq!(budget.active_in_flight, 2);
    assert_eq!(budget.queue_depth, 3);
    assert!(budget.fallback_available);
    assert_eq!(
        budget.decide(),
        AdmissionDecision::Allow {
            lane: PolyglotLane::JuliaCompute,
            remaining_permits: 5,
        }
    );
}

#[test]
fn julia_config_saturates_admission_budget_for_wide_config_values() {
    let config = MemoryJuliaComputeRuntimeConfig {
        max_in_flight_requests: u64::MAX,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let budget = memory_julia_compute_admission_budget(
        &config,
        0,
        0,
        ReadinessState::Ready,
        PressureLevel::Low,
    );

    assert_eq!(budget.max_in_flight, Some(u32::MAX));
}

#[test]
fn runtime_snapshot_projects_route_and_admission_facts() -> Result<(), SnapshotInvariantError> {
    let config = MemoryJuliaComputeRuntimeConfig {
        max_in_flight_requests: 9,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot =
        runtime_polyglot_snapshot(&config, 4, 1, ReadinessState::Ready, PressureLevel::Medium)?;

    assert_eq!(snapshot.route_refs().len(), 1);
    assert_eq!(
        snapshot.route_refs()[0].route,
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE
    );
    assert_eq!(
        snapshot.admission_decision_for_lane(PolyglotLane::JuliaCompute),
        Some(AdmissionDecision::Allow {
            lane: PolyglotLane::JuliaCompute,
            remaining_permits: 5,
        })
    );
    Ok(())
}

#[test]
fn document_extract_pressure_snapshot_projects_supplied_counters()
-> Result<(), SnapshotInvariantError> {
    let pressure = document_extract_pressure_evidence(Some(2), 2, 1, 0, 0, true);

    let snapshot = document_extract_pressure_snapshot(pressure)?;

    assert_eq!(pressure.pressure_level(), PressureLevel::Critical);
    assert_eq!(snapshot.route_refs().len(), 1);
    assert_eq!(
        snapshot.admission_decision_for_lane(PolyglotLane::PythonDocling),
        Some(AdmissionDecision::Reject {
            lane: PolyglotLane::PythonDocling,
            reason: RejectionReason::PressureCritical,
        })
    );
    assert_eq!(
        snapshot
            .evidence_for_lane(PolyglotLane::PythonDocling)
            .map(|evidence| evidence.pressure),
        Some(PressureLevel::Critical)
    );
    Ok(())
}

#[test]
fn document_extract_schedule_plan_uses_orchestrator_policy() {
    let pressure = document_extract_pressure_evidence(Some(6), 2, 0, 0, 0, false);

    let plan = document_extract_schedule_plan(pressure, Some(5), Some(3), 4);

    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.reason, DoclingScheduleReason::CapacityAvailable);
    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
}
