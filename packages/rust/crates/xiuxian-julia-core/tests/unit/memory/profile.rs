use super::{
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID, MEMORY_JULIA_COMPUTE_FAMILY_ID,
    MemoryJuliaComputeProfile, request_schema_id,
};

#[test]
fn parse_memory_julia_compute_profile_recognizes_staged_ids() {
    assert_eq!(
        MemoryJuliaComputeProfile::parse(MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID),
        Some(MemoryJuliaComputeProfile::EpisodicRecall)
    );
    assert_eq!(MemoryJuliaComputeProfile::parse("unknown"), None);
}

#[test]
fn memory_julia_compute_profile_contract_exposes_family_metadata() {
    let profile = MemoryJuliaComputeProfile::MemoryPlanTuning;
    assert_eq!(MEMORY_JULIA_COMPUTE_FAMILY_ID, "memory");
    assert_eq!(profile.capability_id(), "memory_plan_tuning");
    assert_eq!(request_schema_id(profile), "memory.plan_tuning.request.v1");
    assert_eq!(
        MemoryJuliaComputeProfile::MemoryCalibration.default_route(),
        "/memory/calibration"
    );
}
