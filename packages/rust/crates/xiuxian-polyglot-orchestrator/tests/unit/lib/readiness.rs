use crate::{
    AdmissionDecision, BenchmarkState, ContractValidationState, JuliaAcceleratorDiagnostics,
    JuliaReadinessEvidence, JuliaThreadPinningDiagnostics, JuliaThreadPinningState,
    JuliaThreadTopology, LaneCapability, ManifestReadinessState, PolyglotLane, PressureLevel,
    ReadinessState, RejectionReason, WarmupState,
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

#[test]
fn thread_pinning_diagnostics_serialize_without_blocking_readiness() -> Result<(), serde_json::Error>
{
    let diagnostics = JuliaThreadPinningDiagnostics::new(
        JuliaThreadPinningState::Unavailable,
        JuliaThreadTopology::new(8, 12).with_physical_core_count(Some(6)),
    )
    .with_requested_policy("cores")
    .with_applied_policy("none")
    .with_platform("Darwin-arm64")
    .with_notes(["ThreadPinning.jl unavailable on this worker"]);
    let evidence = JuliaReadinessEvidence::graph_search_profile("wendaosearch")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_thread_pinning_diagnostics(diagnostics);

    let serialized = serde_json::to_string(&evidence)?;

    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert!(serialized.contains("thread_pinning_diagnostics"));
    assert!(serialized.contains("unavailable"));
    assert!(serialized.contains("ThreadPinning.jl unavailable"));
    Ok(())
}

#[test]
fn accelerator_diagnostics_serialize_without_blocking_readiness() -> Result<(), serde_json::Error> {
    let diagnostics = vec![
        JuliaAcceleratorDiagnostics::new("metal", true, true).with_observed_output_count(Some(4)),
        JuliaAcceleratorDiagnostics::new("cuda", false, false)
            .with_notes(["CUDA.jl was not loaded by this worker"]),
    ];
    let evidence = JuliaReadinessEvidence::graph_evidence_profile("wendaograph_gnn")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_accelerator_diagnostics(diagnostics);

    let serialized = serde_json::to_string(&evidence)?;

    assert_eq!(evidence.readiness_state(), ReadinessState::Ready);
    assert!(serialized.contains("accelerator_diagnostics"));
    assert!(serialized.contains("metal"));
    assert!(serialized.contains("CUDA.jl was not loaded"));
    Ok(())
}
