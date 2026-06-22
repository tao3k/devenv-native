use crate::wendao_contracts::{
    MemoryJuliaComputeAdmissionBudgetInput, RuntimePolyglotSnapshotInput,
    memory_julia_compute_admission_budget, runtime_polyglot_snapshot,
};
use crate::{
    AdmissionDecision, PolyglotLane, PressureLevel, ReadinessState, SnapshotInvariantError,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;
use xiuxian_wendao_runtime::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

#[test]
fn julia_config_projects_to_admission_budget() {
    let config = MemoryJuliaComputeRuntimeConfig {
        max_in_flight_requests: 7,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let budget = memory_julia_compute_admission_budget(MemoryJuliaComputeAdmissionBudgetInput {
        config: &config,
        active_in_flight: 2,
        queue_depth: 3,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Medium,
    });

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
    let budget = memory_julia_compute_admission_budget(MemoryJuliaComputeAdmissionBudgetInput {
        config: &config,
        active_in_flight: 0,
        queue_depth: 0,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Low,
    });

    assert_eq!(budget.max_in_flight, Some(u32::MAX));
}

#[test]
fn runtime_snapshot_projects_route_and_admission_facts() -> Result<(), SnapshotInvariantError> {
    let config = MemoryJuliaComputeRuntimeConfig {
        max_in_flight_requests: 9,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let snapshot = runtime_polyglot_snapshot(RuntimePolyglotSnapshotInput {
        config: &config,
        active_in_flight: 4,
        queue_depth: 1,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Medium,
    })?;

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
