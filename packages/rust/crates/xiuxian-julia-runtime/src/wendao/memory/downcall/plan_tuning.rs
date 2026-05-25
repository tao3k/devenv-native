//! Downcall helpers for Julia recall-plan tuning requests.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

use crate::wendao::memory::host::{
    MemoryPlanTuningInputs, build_memory_plan_tuning_request_rows_from_inputs,
};
use crate::wendao::memory::{
    MemoryJuliaPlanTuningAdviceRow, fetch_memory_julia_plan_tuning_advice_rows,
};

/// Compose Rust tuning-input staging plus the Julia `memory_plan_tuning`
/// downcall in one plugin-owned helper.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when host input staging fails, the Flight
/// roundtrip fails, or the Julia response cannot be decoded.
pub async fn fetch_plan_tuning_advice_rows_from_inputs(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    inputs: &[MemoryPlanTuningInputs],
) -> Result<Vec<MemoryJuliaPlanTuningAdviceRow>, RepoIntelligenceError> {
    let request_rows = build_memory_plan_tuning_request_rows_from_inputs(inputs)?;
    fetch_memory_julia_plan_tuning_advice_rows(runtime, request_rows.as_slice()).await
}
