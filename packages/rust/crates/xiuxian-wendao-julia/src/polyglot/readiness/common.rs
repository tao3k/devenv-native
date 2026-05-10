//! Shared readiness helpers for Julia profile projections.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, JuliaComputeTaskShape, JuliaReadinessEvidence,
    JuliaSchedulePlan, JuliaSchedulingInput, LaneCapability,
};

use crate::polyglot::state::JuliaProfileSchedulingFacts;

pub(super) fn julia_static_contract_readiness_evidence(
    profile: JuliaStaticContractReadinessProfile,
    warmup: xiuxian_polyglot_orchestrator::WarmupState,
    benchmark: BenchmarkState,
    window: JuliaReadinessWindow,
) -> JuliaReadinessEvidence {
    JuliaReadinessEvidence::new(profile.capability, profile.profile_id)
        .with_schema_version(profile.schema_version)
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(xiuxian_polyglot_orchestrator::ManifestReadinessState::Ready)
        .with_warmup(warmup)
        .with_benchmark(benchmark)
        .with_admission_window(
            window.max_in_flight,
            window.active_in_flight,
            window.queue_depth,
        )
        .with_fallback_available(false)
}

pub(super) fn julia_schedule_plan_from_readiness(
    readiness: JuliaReadinessEvidence,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    JuliaSchedulingInput::new(readiness, shape, facts.runtime_stats)
        .with_fallback_available(facts.fallback_available)
        .with_deadline_ms(facts.deadline_ms)
        .with_target_latency_ms(facts.target_latency_ms)
        .plan()
}

#[derive(Clone, Copy)]
pub(super) struct JuliaStaticContractReadinessProfile {
    pub(super) capability: LaneCapability,
    pub(super) profile_id: &'static str,
    pub(super) schema_version: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct JuliaReadinessWindow {
    pub(super) max_in_flight: Option<u32>,
    pub(super) active_in_flight: u32,
    pub(super) queue_depth: u32,
}

pub(super) fn max_in_flight_as_u32(max_in_flight_requests: u64) -> u32 {
    u32::try_from(max_in_flight_requests).unwrap_or(u32::MAX)
}

pub(super) fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

pub(super) fn latency_ms_as_u32(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    if value < 1.0 {
        return 1;
    }
    if value >= 4_294_967_295.0 {
        return u32::MAX;
    }
    format!("{value:.0}").parse().unwrap_or(u32::MAX)
}
