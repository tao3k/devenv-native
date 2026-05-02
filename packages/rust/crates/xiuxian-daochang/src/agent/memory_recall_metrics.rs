//! Memory recall metric aggregation and summary snapshots.

use num_traits::ToPrimitive;

use crate::{Agent, SessionMemoryRecallDecision};

/// Histogram buckets for end-to-end memory-recall pipeline latency.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryRecallLatencyBucketsSnapshot {
    /// Number of completed pipelines at or below 10 ms.
    pub le_10ms: u64,
    /// Number of completed pipelines above 10 ms and at or below 25 ms.
    pub le_25ms: u64,
    /// Number of completed pipelines above 25 ms and at or below 50 ms.
    pub le_50ms: u64,
    /// Number of completed pipelines above 50 ms and at or below 100 ms.
    pub le_100ms: u64,
    /// Number of completed pipelines above 100 ms and at or below 250 ms.
    pub le_250ms: u64,
    /// Number of completed pipelines above 250 ms and at or below 500 ms.
    pub le_500ms: u64,
    /// Number of completed pipelines above 500 ms.
    pub gt_500ms: u64,
}

/// Process-level memory-recall metrics snapshot used by diagnostics surfaces.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MemoryRecallMetricsSnapshot {
    /// Unix timestamp in milliseconds when the snapshot was captured.
    pub captured_at_unix_ms: u64,
    /// Number of recall plans started.
    pub planned_total: u64,
    /// Number of recall plans that injected memory into context.
    pub injected_total: u64,
    /// Number of recall plans that completed without injection.
    pub skipped_total: u64,
    /// Number of completed recall plans.
    pub completed_total: u64,
    /// Total number of recalled items selected before injection filtering.
    pub selected_total: u64,
    /// Total number of recalled items injected into context.
    pub injected_items_total: u64,
    /// Total character count injected into context from recalled memory.
    pub context_chars_injected_total: u64,
    /// Sum of pipeline duration across completed recall plans.
    pub pipeline_duration_ms_total: u64,
    /// Average end-to-end pipeline duration in milliseconds.
    pub avg_pipeline_duration_ms: f32,
    /// Average number of selected items per completed recall plan.
    pub avg_selected_per_completed: f32,
    /// Average number of injected items for plans that injected memory.
    pub avg_injected_per_injected: f32,
    /// Ratio of injected plans to completed plans in the `0..=1` range.
    pub injected_rate: f32,
    /// Latency histogram for completed recall plans.
    pub latency_buckets: MemoryRecallLatencyBucketsSnapshot,
    /// Number of successful embedding requests made by the recall pipeline.
    pub embedding_success_total: u64,
    /// Number of embedding requests that timed out.
    pub embedding_timeout_total: u64,
    /// Number of embedding requests rejected by cooldown protection.
    pub embedding_cooldown_reject_total: u64,
    /// Number of embedding attempts skipped because embedding was unavailable.
    pub embedding_unavailable_total: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MemoryRecallMetricsState {
    planned_total: u64,
    injected_total: u64,
    skipped_total: u64,
    selected_total: u64,
    injected_items_total: u64,
    context_chars_injected_total: u64,
    pipeline_duration_ms_total: u64,
    latency_buckets: MemoryRecallLatencyBucketsSnapshot,
    embedding_success_total: u64,
    embedding_timeout_total: u64,
    embedding_cooldown_reject_total: u64,
    embedding_unavailable_total: u64,
}

impl MemoryRecallMetricsState {
    pub(crate) fn observe_plan(&mut self) {
        self.planned_total = self.planned_total.saturating_add(1);
    }

    pub(crate) fn observe_result(
        &mut self,
        decision: SessionMemoryRecallDecision,
        recalled_selected: usize,
        recalled_injected: usize,
        context_chars_injected: usize,
        pipeline_duration_ms: u64,
    ) {
        match decision {
            SessionMemoryRecallDecision::Injected => {
                self.injected_total = self.injected_total.saturating_add(1);
            }
            SessionMemoryRecallDecision::Skipped => {
                self.skipped_total = self.skipped_total.saturating_add(1);
            }
        }

        self.selected_total = self.selected_total.saturating_add(recalled_selected as u64);
        self.injected_items_total = self
            .injected_items_total
            .saturating_add(recalled_injected as u64);
        self.context_chars_injected_total = self
            .context_chars_injected_total
            .saturating_add(context_chars_injected as u64);
        self.pipeline_duration_ms_total = self
            .pipeline_duration_ms_total
            .saturating_add(pipeline_duration_ms);
        self.observe_latency_bucket(pipeline_duration_ms);
    }

    fn observe_latency_bucket(&mut self, duration_ms: u64) {
        if duration_ms <= 10 {
            self.latency_buckets.le_10ms = self.latency_buckets.le_10ms.saturating_add(1);
        } else if duration_ms <= 25 {
            self.latency_buckets.le_25ms = self.latency_buckets.le_25ms.saturating_add(1);
        } else if duration_ms <= 50 {
            self.latency_buckets.le_50ms = self.latency_buckets.le_50ms.saturating_add(1);
        } else if duration_ms <= 100 {
            self.latency_buckets.le_100ms = self.latency_buckets.le_100ms.saturating_add(1);
        } else if duration_ms <= 250 {
            self.latency_buckets.le_250ms = self.latency_buckets.le_250ms.saturating_add(1);
        } else if duration_ms <= 500 {
            self.latency_buckets.le_500ms = self.latency_buckets.le_500ms.saturating_add(1);
        } else {
            self.latency_buckets.gt_500ms = self.latency_buckets.gt_500ms.saturating_add(1);
        }
    }

    pub(crate) fn observe_embedding_success(&mut self) {
        self.embedding_success_total = self.embedding_success_total.saturating_add(1);
    }

    pub(crate) fn observe_embedding_timeout(&mut self) {
        self.embedding_timeout_total = self.embedding_timeout_total.saturating_add(1);
    }

    pub(crate) fn observe_embedding_cooldown_reject(&mut self) {
        self.embedding_cooldown_reject_total =
            self.embedding_cooldown_reject_total.saturating_add(1);
    }

    pub(crate) fn observe_embedding_unavailable(&mut self) {
        self.embedding_unavailable_total = self.embedding_unavailable_total.saturating_add(1);
    }

    pub(crate) fn snapshot(self) -> MemoryRecallMetricsSnapshot {
        let completed_total = self.injected_total.saturating_add(self.skipped_total);
        MemoryRecallMetricsSnapshot {
            captured_at_unix_ms: now_unix_ms(),
            planned_total: self.planned_total,
            injected_total: self.injected_total,
            skipped_total: self.skipped_total,
            completed_total,
            selected_total: self.selected_total,
            injected_items_total: self.injected_items_total,
            context_chars_injected_total: self.context_chars_injected_total,
            pipeline_duration_ms_total: self.pipeline_duration_ms_total,
            avg_pipeline_duration_ms: ratio_as_f32(
                self.pipeline_duration_ms_total,
                completed_total,
            ),
            avg_selected_per_completed: ratio_as_f32(self.selected_total, completed_total),
            avg_injected_per_injected: ratio_as_f32(self.injected_items_total, self.injected_total),
            injected_rate: ratio_as_f32(self.injected_total, completed_total),
            latency_buckets: self.latency_buckets,
            embedding_success_total: self.embedding_success_total,
            embedding_timeout_total: self.embedding_timeout_total,
            embedding_cooldown_reject_total: self.embedding_cooldown_reject_total,
            embedding_unavailable_total: self.embedding_unavailable_total,
        }
    }
}

fn ratio_as_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        return 0.0;
    }
    numerator.to_f32().unwrap_or(f32::MAX) / denominator.to_f32().unwrap_or(f32::MAX)
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

impl Agent {
    pub(crate) async fn record_memory_recall_plan_metrics(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_plan();
    }

    pub(crate) async fn record_memory_recall_result_metrics(
        &self,
        decision: SessionMemoryRecallDecision,
        recalled_selected: usize,
        recalled_injected: usize,
        context_chars_injected: usize,
        pipeline_duration_ms: u64,
    ) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_result(
            decision,
            recalled_selected,
            recalled_injected,
            context_chars_injected,
            pipeline_duration_ms,
        );
    }

    pub(crate) async fn record_memory_embedding_success_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_success();
    }

    pub(crate) async fn record_memory_embedding_timeout_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_timeout();
    }

    pub(crate) async fn record_memory_embedding_cooldown_reject_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_cooldown_reject();
    }

    pub(crate) async fn record_memory_embedding_unavailable_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_unavailable();
    }

    /// Returns the current process-level memory-recall metrics snapshot.
    pub async fn inspect_memory_recall_metrics(&self) -> MemoryRecallMetricsSnapshot {
        let guard = self.memory_recall_metrics.read().await;
        (*guard).snapshot()
    }
}
