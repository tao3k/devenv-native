use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use super::capacity::{OcrCapacitySnapshot, OcrSchedulerLane};

const METRIC_WINDOW: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PdfOcrSchedulerSnapshot {
    pub(crate) max_worker_bound: usize,
    pub(crate) current_worker_budget: usize,
    pub(crate) available_worker_permits: usize,
    pub(crate) in_process_workers: usize,
    pub(crate) in_flight_shards: usize,
    pub(crate) cache_hits: u64,
    pub(crate) cache_misses: u64,
    pub(crate) live_requests: u64,
    pub(crate) queue_wait_p50_ms: Option<u64>,
    pub(crate) queue_wait_p95_ms: Option<u64>,
    pub(crate) ocr_latency_p50_ms: Option<u64>,
    pub(crate) ocr_latency_p95_ms: Option<u64>,
    pub(crate) source_pdf_page_range_shards: u64,
    pub(crate) rendered_page_shards: u64,
    pub(crate) rendered_region_shards: u64,
    pub(crate) budget_increase_events: u64,
    pub(crate) budget_decrease_events: u64,
}

#[derive(Debug, Default)]
pub(super) struct PdfOcrSchedulerMetrics {
    state: Mutex<PdfOcrSchedulerMetricState>,
}

#[derive(Debug, Default)]
struct PdfOcrSchedulerMetricState {
    cache_hits: u64,
    cache_misses: u64,
    live_requests: u64,
    queue_wait_ms: VecDeque<u64>,
    ocr_latency_ms: VecDeque<u64>,
    source_pdf_page_range_shards: u64,
    rendered_page_shards: u64,
    rendered_region_shards: u64,
}

impl PdfOcrSchedulerMetrics {
    pub(super) fn record_cache_resolution(&self, hit_count: usize, miss_count: usize) {
        let mut state = self.lock_state();
        state.cache_hits = state.cache_hits.saturating_add(hit_count as u64);
        state.cache_misses = state.cache_misses.saturating_add(miss_count as u64);
    }

    pub(super) fn record_live_request(&self, lane: OcrSchedulerLane, shard_count: usize) {
        let mut state = self.lock_state();
        state.live_requests = state.live_requests.saturating_add(1);
        match lane {
            OcrSchedulerLane::SourcePdfPageRange => {
                state.source_pdf_page_range_shards = state
                    .source_pdf_page_range_shards
                    .saturating_add(shard_count as u64);
            }
            OcrSchedulerLane::RenderedPage => {
                state.rendered_page_shards = state
                    .rendered_page_shards
                    .saturating_add(shard_count as u64);
            }
            OcrSchedulerLane::RenderedRegion => {
                state.rendered_region_shards = state
                    .rendered_region_shards
                    .saturating_add(shard_count as u64);
            }
        }
    }

    pub(super) fn record_queue_wait(&self, duration: Duration) {
        let mut state = self.lock_state();
        push_window(&mut state.queue_wait_ms, duration_to_ms(duration));
    }

    pub(super) fn record_ocr_latency(&self, duration: Duration) {
        let mut state = self.lock_state();
        push_window(&mut state.ocr_latency_ms, duration_to_ms(duration));
    }

    pub(super) fn snapshot(
        &self,
        capacity: &OcrCapacitySnapshot,
        available_worker_permits: usize,
        in_flight_shards: usize,
    ) -> PdfOcrSchedulerSnapshot {
        let state = self.lock_state();
        PdfOcrSchedulerSnapshot {
            max_worker_bound: capacity.max_worker_bound,
            current_worker_budget: capacity.current_worker_budget,
            available_worker_permits,
            in_process_workers: capacity
                .max_worker_bound
                .saturating_sub(available_worker_permits),
            in_flight_shards,
            cache_hits: state.cache_hits,
            cache_misses: state.cache_misses,
            live_requests: state.live_requests,
            queue_wait_p50_ms: percentile(state.queue_wait_ms.iter().copied(), 50),
            queue_wait_p95_ms: percentile(state.queue_wait_ms.iter().copied(), 95),
            ocr_latency_p50_ms: percentile(state.ocr_latency_ms.iter().copied(), 50),
            ocr_latency_p95_ms: percentile(state.ocr_latency_ms.iter().copied(), 95),
            source_pdf_page_range_shards: state.source_pdf_page_range_shards,
            rendered_page_shards: state.rendered_page_shards,
            rendered_region_shards: state.rendered_region_shards,
            budget_increase_events: capacity.budget_increase_events,
            budget_decrease_events: capacity.budget_decrease_events,
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, PdfOcrSchedulerMetricState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

fn push_window(window: &mut VecDeque<u64>, value: u64) {
    if window.len() >= METRIC_WINDOW {
        window.pop_front();
    }
    window.push_back(value);
}

fn duration_to_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn percentile(values: impl Iterator<Item = u64>, percentile: usize) -> Option<u64> {
    let mut values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let percentile = percentile.min(100);
    let index = values
        .len()
        .saturating_sub(1)
        .saturating_mul(percentile)
        .div_ceil(100);
    values.get(index).copied()
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/metrics.rs"]
mod tests;
