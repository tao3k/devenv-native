use std::sync::{Mutex, MutexGuard};

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, is_hosted_vlm_direct_profile};
use xiuxian_wendao_attachments::polyglot::{
    pdf_ocr_shard_pressure_evidence, pdf_ocr_shard_schedule_plan,
    pdf_ocr_source_range_shard_schedule_plan,
};

use super::scheduler::DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV;

const HEALTHY_STREAK_BEFORE_INCREASE: usize = 2;
const PRESSURE_LATENCY_MS: u64 = 60_000;
const PRESSURE_QUEUE_WAIT_MS: u64 = 15_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OcrSchedulerLane {
    SourcePdfPageRange,
    RenderedPage,
    RenderedRegion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OcrCapacitySnapshot {
    pub(super) max_worker_bound: usize,
    pub(super) current_worker_budget: usize,
    pub(super) healthy_streak: usize,
    pub(super) budget_increase_events: u64,
    pub(super) budget_decrease_events: u64,
}

#[derive(Debug)]
pub(super) struct OcrCapacityController {
    state: Mutex<OcrCapacityState>,
}

#[derive(Debug, Clone)]
struct OcrCapacityState {
    max_worker_bound: usize,
    current_worker_budget: usize,
    healthy_streak: usize,
    budget_increase_events: u64,
    budget_decrease_events: u64,
}

impl OcrCapacityController {
    pub(super) fn new(max_worker_bound: usize) -> Self {
        let max_worker_bound = max_worker_bound.max(1);
        Self {
            state: Mutex::new(OcrCapacityState {
                max_worker_bound,
                current_worker_budget: initial_worker_budget(max_worker_bound),
                healthy_streak: 0,
                budget_increase_events: 0,
                budget_decrease_events: 0,
            }),
        }
    }

    #[cfg(test)]
    pub(super) fn new_with_current_budget(
        max_worker_bound: usize,
        current_worker_budget: usize,
    ) -> Self {
        let max_worker_bound = max_worker_bound.max(1);
        Self {
            state: Mutex::new(OcrCapacityState {
                max_worker_bound,
                current_worker_budget: current_worker_budget.clamp(1, max_worker_bound),
                healthy_streak: 0,
                budget_increase_events: 0,
                budget_decrease_events: 0,
            }),
        }
    }

    pub(super) fn budget_for_lane(
        &self,
        shard_count: usize,
        lane: OcrSchedulerLane,
        source_range_override: Option<usize>,
    ) -> usize {
        let snapshot = self.snapshot();
        match lane {
            OcrSchedulerLane::SourcePdfPageRange => scheduled_source_range_worker_budget(
                shard_count,
                snapshot.current_worker_budget,
                snapshot.max_worker_bound,
                source_range_override,
            ),
            OcrSchedulerLane::RenderedPage => scheduled_ocr_worker_budget(
                shard_count,
                snapshot.current_worker_budget,
                snapshot.max_worker_bound,
            ),
            OcrSchedulerLane::RenderedRegion => scheduled_region_worker_budget(
                shard_count,
                snapshot.current_worker_budget,
                snapshot.max_worker_bound,
            ),
        }
    }

    pub(super) fn record_success(&self, queue_wait_ms: u64, latency_ms: u64) {
        if queue_wait_ms > PRESSURE_QUEUE_WAIT_MS || latency_ms > PRESSURE_LATENCY_MS {
            self.record_pressure();
            return;
        }

        let mut state = self.lock_state();
        state.healthy_streak = state.healthy_streak.saturating_add(1);
        if state.healthy_streak >= HEALTHY_STREAK_BEFORE_INCREASE
            && state.current_worker_budget < state.max_worker_bound
        {
            state.current_worker_budget = state.current_worker_budget.saturating_add(1);
            state.healthy_streak = 0;
            state.budget_increase_events = state.budget_increase_events.saturating_add(1);
        }
    }

    pub(super) fn record_failure(&self) {
        self.record_pressure();
    }

    pub(super) fn snapshot(&self) -> OcrCapacitySnapshot {
        let state = self.lock_state();
        OcrCapacitySnapshot {
            max_worker_bound: state.max_worker_bound,
            current_worker_budget: state.current_worker_budget,
            healthy_streak: state.healthy_streak,
            budget_increase_events: state.budget_increase_events,
            budget_decrease_events: state.budget_decrease_events,
        }
    }

    fn record_pressure(&self) {
        let mut state = self.lock_state();
        state.healthy_streak = 0;
        let reduced = state.current_worker_budget.div_ceil(2).max(1);
        if reduced < state.current_worker_budget {
            state.current_worker_budget = reduced;
            state.budget_decrease_events = state.budget_decrease_events.saturating_add(1);
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, OcrCapacityState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

pub(super) fn classify_ocr_lane(inputs: &[PdfOcrShardInput]) -> OcrSchedulerLane {
    if is_source_pdf_page_range_batch(inputs) {
        return OcrSchedulerLane::SourcePdfPageRange;
    }
    if inputs.iter().any(|input| input.shard_type == "region") {
        return OcrSchedulerLane::RenderedRegion;
    }
    OcrSchedulerLane::RenderedPage
}

pub(super) fn source_range_override_from_environment() -> Option<usize> {
    std::env::var(DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
}

#[cfg(test)]
pub(super) fn is_contiguous_source_pdf_page_range(inputs: &[PdfOcrShardInput]) -> bool {
    let Some(first) = inputs.first() else {
        return false;
    };
    if !is_source_pdf_page_range_batch(inputs) {
        return false;
    }
    inputs.iter().enumerate().all(|(offset, input)| {
        input.page_index == first.page_index + u32::try_from(offset).unwrap_or(u32::MAX)
    })
}

pub(super) fn is_source_pdf_page_range_batch(inputs: &[PdfOcrShardInput]) -> bool {
    let Some(first) = inputs.first() else {
        return false;
    };
    if !is_source_pdf_page_input(first, first.source_path.as_str()) {
        return false;
    }
    inputs
        .iter()
        .all(|input| is_source_pdf_page_input(input, first.source_path.as_str()))
}

fn is_source_pdf_page_input(input: &PdfOcrShardInput, source_path: &str) -> bool {
    input.source_path == source_path
        && input.shard_type == "page"
        && !is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
        && input.source_path.to_ascii_lowercase().ends_with(".pdf")
}

fn scheduled_source_range_worker_budget(
    shard_count: usize,
    current_worker_budget: usize,
    max_worker_bound: usize,
    source_range_override: Option<usize>,
) -> usize {
    let shard_count = shard_count.max(1);
    let current_worker_budget = current_worker_budget.max(1);
    let max_worker_bound = max_worker_bound.max(1);
    let pressure = pdf_ocr_shard_pressure_evidence(
        Some(saturating_usize_to_u32(max_worker_bound)),
        0,
        0,
        0,
        0,
        0,
        false,
    );
    let plan = pdf_ocr_source_range_shard_schedule_plan(
        pressure,
        Some(saturating_usize_to_u32(current_worker_budget)),
        source_range_override.map(saturating_usize_to_u32),
        Some(saturating_usize_to_u32(max_worker_bound)),
        saturating_usize_to_u32(shard_count),
    );
    usize::try_from(plan.recommended_workers)
        .unwrap_or(usize::MAX)
        .max(1)
}

fn scheduled_ocr_worker_budget(
    shard_count: usize,
    requested_workers: usize,
    max_worker_bound: usize,
) -> usize {
    let shard_count = shard_count.max(1);
    let requested_workers = requested_workers.max(1);
    let max_worker_bound = max_worker_bound.max(1);
    let pressure = pdf_ocr_shard_pressure_evidence(
        Some(saturating_usize_to_u32(max_worker_bound)),
        0,
        0,
        0,
        0,
        0,
        false,
    );
    let plan = pdf_ocr_shard_schedule_plan(
        pressure,
        Some(saturating_usize_to_u32(requested_workers)),
        Some(saturating_usize_to_u32(max_worker_bound)),
        saturating_usize_to_u32(shard_count),
    );
    usize::try_from(plan.recommended_workers)
        .unwrap_or(usize::MAX)
        .max(1)
}

fn scheduled_region_worker_budget(
    shard_count: usize,
    current_worker_budget: usize,
    max_worker_bound: usize,
) -> usize {
    let shard_count = shard_count.max(1);
    let current_worker_budget = current_worker_budget.max(1);
    let max_worker_bound = max_worker_bound.max(1);
    current_worker_budget
        .saturating_add(ceil_sqrt_usize(shard_count))
        .min(shard_count)
        .min(max_worker_bound)
        .max(1)
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn initial_worker_budget(max_worker_bound: usize) -> usize {
    ceil_sqrt_usize(max_worker_bound.max(1)).max(1)
}

fn ceil_sqrt_usize(value: usize) -> usize {
    if value <= 1 {
        return value;
    }
    let mut root = 1usize;
    while root.saturating_mul(root) < value {
        root = root.saturating_add(1);
    }
    root
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/capacity.rs"]
mod tests;
