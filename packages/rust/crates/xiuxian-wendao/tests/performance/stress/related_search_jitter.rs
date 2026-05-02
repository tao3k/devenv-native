use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::performance::support::{
    PerfBudget, PerfRunConfig, RELATED_LIMIT, RELATED_MAX_DISTANCE, assert_perf_budget,
    build_index, default_ppr_options, env_f64, env_u64, env_usize, run_async_budget, seed_set,
};

const SUITE: &str = "xiuxian-wendao/perf/stress";
const CASE: &str = "related_search_jitter";
const NODE_COUNT: usize = 4_096;
const HUB_COUNT: usize = 64;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "nightly performance-stress gate"]
async fn stress_related_search_jitter_gate() -> Result<(), String> {
    let (_tmp, index) = build_index(NODE_COUNT, HUB_COUNT)?;
    let index = Arc::new(index);
    let ppr = Arc::new(default_ppr_options());
    let seeds = Arc::new(seed_set(NODE_COUNT, 512));
    let cursor = Arc::new(AtomicUsize::new(0));

    let config = PerfRunConfig {
        warmup_samples: env_usize("XIUXIAN_WENDAO_PERF_STRESS_WARMUP", 4),
        samples: env_usize("XIUXIAN_WENDAO_PERF_STRESS_SAMPLES", 36),
        timeout_ms: env_u64("XIUXIAN_WENDAO_PERF_STRESS_TIMEOUT_MS", 600),
        concurrency: env_usize("XIUXIAN_WENDAO_PERF_STRESS_CONCURRENCY", 8),
    };
    let report = run_async_budget(SUITE, CASE, &config, || {
        let index = Arc::clone(&index);
        let ppr = Arc::clone(&ppr);
        let seeds = Arc::clone(&seeds);
        let cursor = Arc::clone(&cursor);
        async move {
            tokio::task::yield_now().await;
            let position = cursor.fetch_add(1, Ordering::Relaxed) % seeds.len();
            let seed = &seeds[position];
            let (rows, diagnostics) = index.related_with_diagnostics(
                seed,
                RELATED_MAX_DISTANCE,
                RELATED_LIMIT,
                Some(&ppr),
            );
            if rows.is_empty() {
                return Err(format!("expected related rows for seed={seed}"));
            }
            if diagnostics.is_none() {
                return Err(format!("missing diagnostics for seed={seed}"));
            }
            Ok::<(), String>(())
        }
    })
    .await;

    let budget = PerfBudget {
        max_p95_latency_ms: Some(env_f64("XIUXIAN_WENDAO_PERF_STRESS_P95_MS", 180.0)),
        min_throughput_qps: Some(env_f64("XIUXIAN_WENDAO_PERF_STRESS_MIN_QPS", 30.0)),
        max_error_rate: Some(env_f64("XIUXIAN_WENDAO_PERF_STRESS_MAX_ERROR_RATE", 0.02)),
        ..PerfBudget::new()
    };
    assert_perf_budget(&report, &budget);
    Ok(())
}
