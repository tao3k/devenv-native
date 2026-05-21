//! Public report types for `WendaoSearch` managed service integration helpers.

use std::time::Duration;

/// Result summary for one graph-structural release prewarm probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralPrewarmReport {
    /// Number of logical Flight route calls performed.
    pub route_count: usize,
    /// Total elapsed time for all logical prewarm route calls.
    pub elapsed: Duration,
    /// Stable tiny candidate id used by the solver-demo prewarm request.
    pub candidate_id: String,
}

/// Warm-path timing statistics for a graph-structural release gate.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralWarmPathStats {
    /// Number of measured warm-path samples.
    pub sample_count: usize,
    /// Minimum observed elapsed milliseconds.
    pub min_ms: f64,
    /// Median observed elapsed milliseconds.
    pub median_ms: f64,
    /// P95 observed elapsed milliseconds.
    pub p95_ms: f64,
    /// Maximum observed elapsed milliseconds.
    pub max_ms: f64,
    /// `max_ms / min_ms`, or `0.0` when the minimum is effectively zero.
    pub spread_ratio: f64,
}

/// Stability limits for the graph-structural release gate.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralStabilizationLimits {
    /// Sequential and concurrent warm samples to measure after release prewarm.
    pub sample_count: usize,
    /// Maximum allowed warm-path p95 in milliseconds.
    pub max_p95_ms: f64,
    /// Maximum allowed warm-path max latency in milliseconds.
    pub max_max_ms: f64,
    /// Maximum allowed warm-path spread ratio once latency reaches the
    /// meaningful tail budget.
    pub max_spread_ratio: f64,
    /// Initial in-flight budget when the warm path is stable.
    pub preferred_max_in_flight: usize,
    /// Initial in-flight budget when the warm path has tail instability.
    pub degraded_max_in_flight: usize,
}

/// Stability reason emitted by the graph-structural release gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WendaoSearchGraphStructuralStabilizationReason {
    /// Sequential and concurrent samples stayed within the configured tail
    /// budget.
    Stable,
    /// Sequential samples crossed the configured tail budget.
    SequentialExceeded,
    /// Concurrent samples crossed the configured tail budget.
    ConcurrentExceeded,
    /// Both sequential and concurrent samples crossed the configured tail
    /// budget.
    BothExceeded,
}

impl Default for WendaoSearchGraphStructuralStabilizationLimits {
    fn default() -> Self {
        Self {
            sample_count: 3,
            max_p95_ms: 150.0,
            max_max_ms: 250.0,
            max_spread_ratio: 16.0,
            preferred_max_in_flight: 4,
            degraded_max_in_flight: 1,
        }
    }
}

impl WendaoSearchGraphStructuralStabilizationLimits {
    /// Returns a copy with a bounded non-zero sample count.
    #[must_use]
    pub fn with_sample_count(mut self, sample_count: usize) -> Self {
        self.sample_count = sample_count.max(1);
        self
    }
}

/// Release-gate report for a graph-structural Julia pod.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralStabilizationReport {
    /// The all-route first release prewarm report.
    pub prewarm: WendaoSearchGraphStructuralPrewarmReport,
    /// Sequential warm-path stats after release prewarm.
    pub sequential: WendaoSearchGraphStructuralWarmPathStats,
    /// Concurrent warm-path stats after release prewarm.
    pub concurrent: WendaoSearchGraphStructuralWarmPathStats,
    /// Whether both warm paths passed the configured limits.
    pub stable: bool,
    /// Why this report selected the recommended admission budget.
    pub stability_reason: WendaoSearchGraphStructuralStabilizationReason,
    /// Recommended initial Rust admission budget for this Julia pod.
    pub recommended_max_in_flight: usize,
}
