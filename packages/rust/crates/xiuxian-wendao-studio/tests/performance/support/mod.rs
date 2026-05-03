pub(crate) mod gateway;
mod perf;
pub(crate) mod repo_index_audit;

pub(crate) use perf::{
    PerfBudget, PerfReport, PerfRunConfig, assert_perf_budget, run_async_budget, run_sync_budget,
};

pub(crate) fn env_f64(name: &str, default_value: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default_value)
}

pub(crate) fn env_u64(name: &str, default_value: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default_value)
}

pub(crate) fn env_usize(name: &str, default_value: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default_value)
}
