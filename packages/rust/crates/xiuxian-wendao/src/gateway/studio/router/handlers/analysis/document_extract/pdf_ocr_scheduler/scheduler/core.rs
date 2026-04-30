use std::sync::Arc;

use tokio::sync::Semaphore;

use super::limit::pdf_ocr_worker_limit;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_cache::PdfOcrShardCache;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::capacity::OcrCapacityController;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::inflight::InFlightShardRegistry;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_scheduler::metrics::{
    PdfOcrSchedulerMetrics, PdfOcrSchedulerSnapshot,
};

#[derive(Debug)]
pub(in crate::gateway::studio::router::handlers::analysis::document_extract) struct PdfOcrWorkerScheduler
{
    pub(super) permits: Arc<Semaphore>,
    pub(super) worker_limit: usize,
    pub(super) capacity: OcrCapacityController,
    pub(super) cache: PdfOcrShardCache,
    pub(super) inflight: InFlightShardRegistry,
    pub(super) metrics: PdfOcrSchedulerMetrics,
}

impl PdfOcrWorkerScheduler {
    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn from_environment()
    -> Self {
        Self::with_limit(pdf_ocr_worker_limit())
    }

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn with_limit(
        worker_limit: usize,
    ) -> Self {
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
        }
    }

    pub(super) fn available_permits(&self) -> usize {
        self.permits.available_permits()
    }

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn snapshot(
        &self,
    ) -> PdfOcrSchedulerSnapshot {
        let capacity = self.capacity.snapshot();
        self.metrics
            .snapshot(&capacity, self.available_permits(), self.inflight.len())
    }

    #[cfg(test)]
    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn permits_for_tests(
        &self,
    ) -> Arc<Semaphore> {
        Arc::clone(&self.permits)
    }
}
