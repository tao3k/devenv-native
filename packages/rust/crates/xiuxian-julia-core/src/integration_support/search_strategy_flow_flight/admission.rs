//! Polyglot admission for `SearchStrategyFlow` Flight materialization waves.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, JuliaComputeTaskShape, JuliaRuntimeStats, JuliaScheduleAction,
    JuliaSchedulePlan, JuliaTaskComplexityClass, WarmupState,
};

use crate::polyglot::{JuliaProfileSchedulingFacts, wendao_graph_link_evidence_schedule_plan};

use super::config::SearchStrategyFlowFlightMaterializationConfig;

const FLIGHT_MATERIALIZATION_BATCHABILITY_KEY: &str =
    "wendaograph:search_strategy_flow:flight_materialization";

pub(super) fn route_materialization_wave_size(
    route_count: usize,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<usize, String> {
    if route_count == 0 {
        return Ok(0);
    }
    let schedule_plan = search_strategy_flow_materialization_schedule_plan(route_count, config);
    match schedule_plan.action {
        JuliaScheduleAction::Dispatch | JuliaScheduleAction::Queue => {}
        JuliaScheduleAction::Fallback | JuliaScheduleAction::Reject => {
            return Err(format!(
                "SearchStrategyFlow Flight materialization was rejected by xiuxian-polyglot-orchestrator: action={:?}, reason={:?}",
                schedule_plan.action, schedule_plan.reason
            ));
        }
    }
    let recommended = usize::try_from(schedule_plan.max_in_flight_recommendation)
        .unwrap_or(usize::MAX)
        .max(1);
    Ok(recommended.min(route_count))
}

pub(super) fn search_strategy_flow_materialization_schedule_plan(
    route_count: usize,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> JuliaSchedulePlan {
    let route_count = saturating_usize_to_u32(route_count.max(1));
    let estimated_route_bytes = u64::from(route_count).saturating_mul(64 * 1024);
    let task_shape = JuliaComputeTaskShape::new()
        .with_rows(route_count)
        .with_graph_size(route_count, route_count.saturating_mul(4))
        .with_feature_columns(4)
        .with_byte_size(estimated_route_bytes)
        .with_batchability_key(FLIGHT_MATERIALIZATION_BATCHABILITY_KEY)
        .with_complexity(JuliaTaskComplexityClass::Balanced);
    let runtime_stats = JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired);
    let facts = JuliaProfileSchedulingFacts::new(runtime_stats)
        .with_target_latency_ms(Some(saturating_timeout_millis(config.timeout_seconds)));
    wendao_graph_link_evidence_schedule_plan(task_shape, facts)
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn saturating_timeout_millis(timeout_seconds: u64) -> u32 {
    let timeout_millis = timeout_seconds.saturating_mul(1_000);
    u32::try_from(timeout_millis).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/admission.rs"]
mod tests;
