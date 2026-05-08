use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tokio::sync::Semaphore;

use super::limit::pdf_ocr_worker_limit;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_cache::PdfOcrShardCache;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::capacity::OcrCapacityController;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::inflight::InFlightShardRegistry;
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::metrics::{
    PdfOcrSchedulerMetrics, PdfOcrSchedulerSnapshot,
};

#[derive(Debug)]
pub(crate) struct PdfOcrWorkerScheduler {
    pub(super) permits: Arc<Semaphore>,
    pub(super) worker_limit: usize,
    pub(super) capacity: OcrCapacityController,
    pub(super) cache: PdfOcrShardCache,
    pub(super) inflight: InFlightShardRegistry,
    pub(super) metrics: PdfOcrSchedulerMetrics,
    endpoint_request_cursor: AtomicUsize,
}

impl PdfOcrWorkerScheduler {
    pub(crate) fn from_environment() -> Self {
        Self::with_limit(pdf_ocr_worker_limit())
    }

    pub(crate) fn with_limit(worker_limit: usize) -> Self {
        Self::with_limit_and_cache(worker_limit, PdfOcrShardCache::from_environment())
    }

    pub(super) fn with_limit_and_cache(worker_limit: usize, cache: PdfOcrShardCache) -> Self {
        let worker_limit = worker_limit.max(1);
        Self {
            permits: Arc::new(Semaphore::new(worker_limit)),
            worker_limit,
            capacity: OcrCapacityController::new(worker_limit),
            cache,
            inflight: InFlightShardRegistry::default(),
            metrics: PdfOcrSchedulerMetrics::default(),
            endpoint_request_cursor: AtomicUsize::new(0),
        }
    }

    pub(super) fn available_permits(&self) -> usize {
        self.permits.available_permits()
    }

    pub(crate) fn snapshot(&self) -> PdfOcrSchedulerSnapshot {
        let capacity = self.capacity.snapshot();
        self.metrics
            .snapshot(&capacity, self.available_permits(), self.inflight.len())
    }

    pub(super) fn endpoint_index_for_next_request(
        &self,
        endpoint_count: usize,
    ) -> Result<usize, String> {
        let request_index = self.endpoint_request_cursor.fetch_add(1, Ordering::Relaxed);
        super::dispatch::endpoint_index_for_request(request_index, endpoint_count)
    }

    #[cfg(test)]
    pub(crate) fn permits_for_tests(&self) -> Arc<Semaphore> {
        Arc::clone(&self.permits)
    }
}
