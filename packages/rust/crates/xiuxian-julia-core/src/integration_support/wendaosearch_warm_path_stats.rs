//! Warm-path statistics helpers for managed Julia services.

use std::time::Duration;

use super::{
    WendaoSearchGraphStructuralStabilizationLimits, WendaoSearchGraphStructuralWarmPathStats,
};

pub(super) fn warm_path_stats_from_samples(
    samples: &[Duration],
) -> WendaoSearchGraphStructuralWarmPathStats {
    let mut elapsed_values: Vec<f64> = samples
        .iter()
        .map(|sample| sample.as_secs_f64() * 1000.0)
        .collect();
    elapsed_values.sort_by(f64::total_cmp);
    let min_ms = elapsed_values[0];
    let median_ms = percentile_from_sorted_values(&elapsed_values, 500);
    let p95_ms = percentile_from_sorted_values(&elapsed_values, 950);
    let max_ms = elapsed_values[elapsed_values.len() - 1];
    let spread_ratio = if min_ms <= f64::EPSILON {
        0.0
    } else {
        max_ms / min_ms
    };
    WendaoSearchGraphStructuralWarmPathStats {
        sample_count: elapsed_values.len(),
        min_ms,
        median_ms,
        p95_ms,
        max_ms,
        spread_ratio,
    }
}

fn percentile_from_sorted_values(sorted_values: &[f64], percentile_per_mille: usize) -> f64 {
    let last_index = sorted_values.len() - 1;
    let index = (last_index * percentile_per_mille).div_ceil(1000);
    sorted_values[index]
}

pub(crate) fn warm_path_passes_limits(
    stats: &WendaoSearchGraphStructuralWarmPathStats,
    limits: &WendaoSearchGraphStructuralStabilizationLimits,
) -> bool {
    if stats.p95_ms > limits.max_p95_ms || stats.max_ms > limits.max_max_ms {
        return false;
    }

    // A high spread ratio on tiny millisecond samples is not user-visible by
    // itself. Treat spread as a secondary gate only after max latency enters
    // the p95 budget region.
    stats.max_ms < limits.max_p95_ms || stats.spread_ratio <= limits.max_spread_ratio
}
