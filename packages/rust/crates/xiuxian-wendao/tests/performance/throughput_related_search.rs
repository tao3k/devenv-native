use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::{PerfBudget, PerfRunConfig, assert_perf_budget, run_sync_budget};
use serial_test::file_serial;

use super::support::{
    RELATED_LIMIT, RELATED_MAX_DISTANCE, build_index, default_ppr_options, env_f64, env_u64,
    env_usize, seed_set,
};

const SUITE: &str = "xiuxian-wendao/perf";
const CASE: &str = "throughput_related_search_qps";
const NODE_COUNT: usize = 1_024;
const HUB_COUNT: usize = 16;

#[test]
#[file_serial(wendao_perf_gate)]
fn throughput_related_search_qps_gate() -> Result<(), String> {
    let (_tmp, index) = build_index(NODE_COUNT, HUB_COUNT)?;
    let ppr = default_ppr_options();
    let seeds = seed_set(NODE_COUNT, 96);
    let cursor = AtomicUsize::new(0);

    let config = PerfRunConfig {
        warmup_samples: env_usize("XIUXIAN_WENDAO_PERF_TP_WARMUP", 2),
        samples: env_usize("XIUXIAN_WENDAO_PERF_TP_SAMPLES", 20),
        timeout_ms: env_u64("XIUXIAN_WENDAO_PERF_TP_TIMEOUT_MS", 300),
        concurrency: env_usize("XIUXIAN_WENDAO_PERF_TP_CONCURRENCY", 2),
    };
    let report = run_sync_budget(SUITE, CASE, &config, || -> Result<(), String> {
        let position = cursor.fetch_add(1, Ordering::Relaxed) % seeds.len();
        let seed = &seeds[position];
        let (rows, diagnostics) =
            index.related_with_diagnostics(seed, RELATED_MAX_DISTANCE, RELATED_LIMIT, Some(&ppr));
        if rows.is_empty() {
            return Err(format!("expected related rows for seed={seed}"));
        }
        if diagnostics.is_none() {
            return Err(format!("missing diagnostics for seed={seed}"));
        }
        Ok(())
    });

    let budget = PerfBudget {
        min_throughput_qps: Some(env_f64(
            "XIUXIAN_WENDAO_PERF_TP_MIN_QPS",
            if std::env::var_os("CI").is_some() {
                40.0
            } else {
                25.0
            },
        )),
        max_error_rate: Some(env_f64("XIUXIAN_WENDAO_PERF_TP_MAX_ERROR_RATE", 0.001)),
        ..PerfBudget::new()
    };
    assert_perf_budget(&report, &budget);
    Ok(())
}
