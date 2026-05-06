use crate::{
    AdmissionDecision, BenchmarkState, ContractValidationState, JuliaReadinessEvidence,
    LaneCapability, ManifestReadinessState, PolyglotLane, PressureLevel, ReadinessState,
    RejectionReason, WarmupState,
};

#[test]
fn ready_memory_profile_projects_to_admission_budget() {
    let evidence = JuliaReadinessEvidence::memory_profile("episodic_recall")
        .with_schema_version("memory-family-v1")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_admission_window(Some(4), 1, 0)
        .with_fallback_available(true);

    let budget = evidence.to_admission_budget();

    assert_eq!(evidence.lane, PolyglotLane::JuliaCompute);
    assert_eq!(evidence.capability, LaneCapability::MemoryProfileCompute);
    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert_eq!(evidence.pressure_level(), PressureLevel::Low);
    assert_eq!(
        budget.decide(),
        AdmissionDecision::Allow {
            lane: PolyglotLane::JuliaCompute,
            remaining_permits: 3,
        }
    );
}

#[test]
fn warming_profile_queues_work() {
    let evidence = JuliaReadinessEvidence::memory_profile("gate_scoring")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Warming)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_admission_window(Some(4), 0, 2);

    assert_eq!(evidence.readiness_state(), ReadinessState::Warming);
    assert!(matches!(
        evidence.to_admission_budget().decide(),
        AdmissionDecision::Queue { .. }
    ));
}

#[test]
fn invalid_schema_disables_lane_admission() {
    let evidence = JuliaReadinessEvidence::memory_profile("plan_tuning")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Invalid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold);

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
fn readiness_evidence_serializes_profile_and_states() -> Result<(), serde_json::Error> {
    let evidence = JuliaReadinessEvidence::memory_profile("calibration")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::AboveThreshold);

    let serialized = serde_json::to_string(&evidence)?;

    assert!(serialized.contains("calibration"));
    assert!(serialized.contains("above_threshold"));
    assert_eq!(
        evidence.to_lane_evidence().readiness,
        ReadinessState::Degraded
    );
    Ok(())
}
