//! Coordinates the Studio analysis document extract pdf ocr scheduler branch and keeps its child modules behind one documented reasoning-tree boundary.

mod capacity;
mod config;
mod endpoints;
mod inflight;
mod metrics;
#[path = "scheduler/mod.rs"]
mod scheduler;

#[cfg(test)]
pub(super) use capacity::OcrSchedulerLane;
pub(super) use config::DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV;
pub(crate) use endpoints::pdf_ocr_endpoint_urls;
pub(super) use scheduler::PdfOcrWorkerScheduler;
