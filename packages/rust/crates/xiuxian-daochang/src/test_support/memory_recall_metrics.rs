//! Memory recall metrics helpers exposed for integration tests.

use num_traits::ToPrimitive;

use crate::SessionMemoryRecallDecision;
use crate::agent::memory_recall_metrics::{MemoryRecallMetricsSnapshot, MemoryRecallMetricsState};

#[must_use]
/// Computes a saturating `f32` ratio for metrics tests.
pub fn ratio_as_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        return 0.0;
    }
    numerator.to_f32().unwrap_or(f32::MAX) / denominator.to_f32().unwrap_or(f32::MAX)
}

#[derive(Default)]
/// Test-facing memory recall metrics accumulator.
pub struct TestMemoryRecallMetricsState {
    inner: MemoryRecallMetricsState,
}

impl TestMemoryRecallMetricsState {
    /// Records one recall plan.
    pub fn observe_plan(&mut self) {
        self.inner.observe_plan();
    }

    /// Records one recall result.
    pub fn observe_result(
        &mut self,
        decision: SessionMemoryRecallDecision,
        recalled_selected: usize,
        recalled_injected: usize,
        context_chars_injected: usize,
        pipeline_duration_ms: u64,
    ) {
        self.inner.observe_result(
            decision,
            recalled_selected,
            recalled_injected,
            context_chars_injected,
            pipeline_duration_ms,
        );
    }

    /// Records one embedding success.
    pub fn observe_embedding_success(&mut self) {
        self.inner.observe_embedding_success();
    }

    /// Records one embedding timeout.
    pub fn observe_embedding_timeout(&mut self) {
        self.inner.observe_embedding_timeout();
    }

    /// Records one embedding cooldown rejection.
    pub fn observe_embedding_cooldown_reject(&mut self) {
        self.inner.observe_embedding_cooldown_reject();
    }

    /// Records one embedding unavailable event.
    pub fn observe_embedding_unavailable(&mut self) {
        self.inner.observe_embedding_unavailable();
    }

    #[must_use]
    /// Returns the current metrics snapshot.
    pub fn snapshot(self) -> MemoryRecallMetricsSnapshot {
        self.inner.snapshot()
    }
}
