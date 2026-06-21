//! Host-side staging for Julia recall-plan tuning request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use crate::wendao::memory::{
    MemoryJuliaPlanTuningRequestRow, build_memory_julia_plan_tuning_request_batch,
};

use super::staging::{optional_text, positive_u32_from_usize, required_text, validate_probability};
use super::types::{RecallPlanTuning, normalize_feedback_bias};

const SURFACE: &str = "memory Julia memory_plan_tuning host staging";

/// Host-owned tuning inputs for one Julia `memory_plan_tuning` downcall.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanTuningInputs {
    /// Logical scope of the tuning context.
    pub scope: String,
    /// Optional scenario pack forwarded into the Julia compute lane.
    pub scenario_pack: Option<String>,
    /// Canonical Rust-owned tuning parameters for the current recall plan.
    pub current_plan: RecallPlanTuning,
    /// Host feedback bias to normalize before crossing the Arrow boundary.
    pub feedback_bias: f32,
    /// Recent success rate in `[0, 1]`.
    pub recent_success_rate: f32,
    /// Recent failure rate in `[0, 1]`.
    pub recent_failure_rate: f32,
    /// Recent latency budget in milliseconds.
    pub recent_latency_ms: u64,
}

/// Build typed Julia `memory_plan_tuning` request rows from Rust-owned tuning
/// inputs.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any host tuning input violates the
/// staged `memory_plan_tuning` request contract.
pub fn build_memory_plan_tuning_request_rows_from_inputs(
    inputs: &[MemoryPlanTuningInputs],
) -> Result<Vec<MemoryJuliaPlanTuningRequestRow>, RepoIntelligenceError> {
    inputs.iter().map(build_request_row).collect()
}

/// Build one Julia `memory_plan_tuning` request batch from Rust-owned tuning
/// inputs.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the inputs are empty or any staged
/// row violates the Julia `memory_plan_tuning` request contract.
pub fn build_memory_plan_tuning_request_batch_from_inputs(
    inputs: &[MemoryPlanTuningInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = build_memory_plan_tuning_request_rows_from_inputs(inputs)?;
    if rows.is_empty() {
        return Err(staging_error(
            "memory Julia memory_plan_tuning host staging requires at least one input row",
        ));
    }
    build_memory_julia_plan_tuning_request_batch(&rows)
}

fn build_request_row(
    input: &MemoryPlanTuningInputs,
) -> Result<MemoryJuliaPlanTuningRequestRow, RepoIntelligenceError> {
    let scope = required_text(&input.scope, "scope", SURFACE)?;
    let scenario_pack = optional_text(input.scenario_pack.as_deref());
    let current_k1 = positive_u32_from_usize(input.current_plan.k1, "current_k1", SURFACE)?;
    let current_k2 = positive_u32_from_usize(input.current_plan.k2, "current_k2", SURFACE)?;
    if current_k2 > current_k1 {
        return Err(staging_error(
            "requires current_k2 to be less than or equal to current_k1",
        ));
    }

    validate_probability("current_lambda", input.current_plan.lambda, SURFACE)?;
    validate_probability("current_min_score", input.current_plan.min_score, SURFACE)?;
    let current_max_context_chars = positive_u32_from_usize(
        input.current_plan.max_context_chars,
        "current_max_context_chars",
        SURFACE,
    )?;
    let feedback_bias = normalize_feedback_bias(input.feedback_bias);
    validate_probability("recent_success_rate", input.recent_success_rate, SURFACE)?;
    validate_probability("recent_failure_rate", input.recent_failure_rate, SURFACE)?;
    let combined_rate = input.recent_success_rate + input.recent_failure_rate;
    if combined_rate > 1.0 + f32::EPSILON {
        return Err(staging_error(
            "requires recent_success_rate and recent_failure_rate to sum to at most 1.0",
        ));
    }

    Ok(MemoryJuliaPlanTuningRequestRow {
        scope,
        scenario_pack,
        current_k1,
        current_k2,
        current_lambda: input.current_plan.lambda,
        current_min_score: input.current_plan.min_score,
        current_max_context_chars,
        feedback_bias,
        recent_success_rate: input.recent_success_rate,
        recent_failure_rate: input.recent_failure_rate,
        recent_latency_ms: input.recent_latency_ms,
    })
}

fn staging_error(message: impl Into<String>) -> RepoIntelligenceError {
    super::staging::staging_error(SURFACE, message)
}
