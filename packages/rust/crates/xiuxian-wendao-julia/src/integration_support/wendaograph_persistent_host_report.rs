//! Release report types for persistent `SearchStrategyFlow` Julia hosts.

use std::time::Duration;

use serde_json::{Value, json};

/// Warm-path timing statistics for a persistent `SearchStrategyFlow` Julia host.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchStrategyFlowPersistentHostWarmPathStats {
    /// Number of measured warm submits.
    pub sample_count: usize,
    /// Minimum observed submit time in milliseconds.
    pub min_ms: f64,
    /// Median observed submit time in milliseconds.
    pub median_ms: f64,
    /// P95 observed submit time in milliseconds.
    pub p95_ms: f64,
    /// Maximum observed submit time in milliseconds.
    pub max_ms: f64,
    /// `max_ms / min_ms`, or `0.0` when the minimum is effectively zero.
    pub spread_ratio: f64,
}

/// Stability limits for releasing a persistent `SearchStrategyFlow` Julia host.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchStrategyFlowPersistentHostStabilizationLimits {
    /// Number of warm submit samples to measure after the first prewarm submit.
    pub sample_count: usize,
    /// Maximum allowed warm-path p95 in milliseconds.
    pub max_p95_ms: f64,
    /// Maximum allowed warm-path max latency in milliseconds.
    pub max_max_ms: f64,
    /// Maximum allowed spread ratio once latency reaches the p95 budget region.
    pub max_spread_ratio: f64,
    /// Initial in-flight budget when the warm path is stable.
    pub preferred_max_in_flight: usize,
    /// Initial in-flight budget when the warm path is unstable.
    pub degraded_max_in_flight: usize,
}

/// Stability reason emitted by the `SearchStrategyFlow` persistent host release gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchStrategyFlowPersistentHostStabilizationReason {
    /// Warm submits stayed within the configured tail budget.
    Stable,
    /// Warm submit p95 crossed the configured budget.
    P95Exceeded,
    /// Warm submit max latency crossed the configured budget.
    MaxExceeded,
    /// Warm submit spread crossed the configured budget after entering the
    /// meaningful tail-latency region.
    SpreadExceeded,
}

impl SearchStrategyFlowPersistentHostStabilizationReason {
    /// Returns the stable JSON/reporting token for this stabilization reason.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SearchStrategyFlowPersistentHostStabilizationReason::Stable => "stable",
            SearchStrategyFlowPersistentHostStabilizationReason::P95Exceeded => "p95_exceeded",
            SearchStrategyFlowPersistentHostStabilizationReason::MaxExceeded => "max_exceeded",
            SearchStrategyFlowPersistentHostStabilizationReason::SpreadExceeded => {
                "spread_exceeded"
            }
        }
    }
}

/// Release-gate report for one persistent `SearchStrategyFlow` Julia host.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchStrategyFlowPersistentHostStabilizationReport {
    /// The first real Flight-backed submit used to prewarm the host.
    pub prewarm_elapsed: Duration,
    /// Warm submit timing statistics after prewarm.
    pub warm: SearchStrategyFlowPersistentHostWarmPathStats,
    /// Whether the warm path passed the configured limits.
    pub stable: bool,
    /// Why this report selected the recommended admission budget.
    pub stability_reason: SearchStrategyFlowPersistentHostStabilizationReason,
    /// Recommended initial Rust admission budget for this host.
    pub recommended_max_in_flight: usize,
}

impl SearchStrategyFlowPersistentHostWarmPathStats {
    /// Returns the warm-path timing stats as a stable JSON evidence object.
    #[must_use]
    pub fn to_json_value(&self) -> Value {
        json!({
            "sampleCount": self.sample_count,
            "minMs": self.min_ms,
            "medianMs": self.median_ms,
            "p95Ms": self.p95_ms,
            "maxMs": self.max_ms,
            "spreadRatio": self.spread_ratio,
        })
    }
}

impl SearchStrategyFlowPersistentHostStabilizationReport {
    /// Returns this release report as a stable JSON evidence object.
    #[must_use]
    pub fn to_json_value(&self) -> Value {
        json!({
            "prewarmElapsedMs": self.prewarm_elapsed.as_secs_f64() * 1000.0,
            "warm": self.warm.to_json_value(),
            "stable": self.stable,
            "stabilityReason": self.stability_reason.as_str(),
            "recommendedMaxInFlight": self.recommended_max_in_flight,
        })
    }
}

impl Default for SearchStrategyFlowPersistentHostStabilizationLimits {
    fn default() -> Self {
        Self {
            sample_count: 2,
            max_p95_ms: 750.0,
            max_max_ms: 1_000.0,
            max_spread_ratio: 16.0,
            preferred_max_in_flight: 1,
            degraded_max_in_flight: 1,
        }
    }
}

impl SearchStrategyFlowPersistentHostStabilizationLimits {
    /// Returns a copy with a bounded non-zero sample count.
    #[must_use]
    pub fn with_sample_count(mut self, sample_count: usize) -> Self {
        self.sample_count = sample_count.max(1);
        self
    }

    /// Returns the stability reason for measured warm-path stats.
    #[must_use]
    pub fn stability_reason_for(
        &self,
        stats: &SearchStrategyFlowPersistentHostWarmPathStats,
    ) -> SearchStrategyFlowPersistentHostStabilizationReason {
        if stats.p95_ms > self.max_p95_ms {
            return SearchStrategyFlowPersistentHostStabilizationReason::P95Exceeded;
        }
        if stats.max_ms > self.max_max_ms {
            return SearchStrategyFlowPersistentHostStabilizationReason::MaxExceeded;
        }
        if stats.max_ms >= self.max_p95_ms && stats.spread_ratio > self.max_spread_ratio {
            return SearchStrategyFlowPersistentHostStabilizationReason::SpreadExceeded;
        }
        SearchStrategyFlowPersistentHostStabilizationReason::Stable
    }

    pub(super) fn recommended_max_in_flight_for(
        &self,
        reason: SearchStrategyFlowPersistentHostStabilizationReason,
    ) -> usize {
        match reason {
            SearchStrategyFlowPersistentHostStabilizationReason::Stable => {
                self.preferred_max_in_flight
            }
            SearchStrategyFlowPersistentHostStabilizationReason::P95Exceeded
            | SearchStrategyFlowPersistentHostStabilizationReason::MaxExceeded
            | SearchStrategyFlowPersistentHostStabilizationReason::SpreadExceeded => {
                self.degraded_max_in_flight
            }
        }
        .max(1)
    }
}

pub(super) fn warm_path_stats_from_samples(
    samples: &[Duration],
) -> SearchStrategyFlowPersistentHostWarmPathStats {
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
    SearchStrategyFlowPersistentHostWarmPathStats {
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
