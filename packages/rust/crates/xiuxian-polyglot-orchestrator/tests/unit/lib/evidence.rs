use crate::{
    FallbackEvidence, HealthState, LaneEvidence, LaneEvidenceInput, PolyglotLane, PressureLevel,
    ReadinessState,
};

#[test]
fn critical_pressure_rejects_new_work() {
    assert!(PressureLevel::Critical.rejects_new_work());
    assert!(!PressureLevel::High.rejects_new_work());
}

#[test]
fn readiness_states_identify_normal_traffic() {
    assert!(ReadinessState::Ready.accepts_normal_traffic());
    assert!(ReadinessState::Degraded.accepts_normal_traffic());
    assert!(!ReadinessState::Warming.accepts_normal_traffic());
}

#[test]
fn lane_evidence_serializes_lane_and_states() -> Result<(), serde_json::Error> {
    let evidence = LaneEvidence::new(LaneEvidenceInput {
        lane: PolyglotLane::JuliaCompute,
        health: HealthState::Healthy,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Low,
        fallback: FallbackEvidence::new(false),
    });
    let serialized = serde_json::to_string(&evidence)?;
    assert!(serialized.contains("julia_compute"));
    assert!(serialized.contains("healthy"));
    assert!(serialized.contains("ready"));
    Ok(())
}
