mod capacity;
mod endpoints;
mod inflight;
mod metrics;
#[path = "scheduler/mod.rs"]
mod scheduler;

pub(crate) use endpoints::pdf_ocr_endpoint_urls;
pub(super) use scheduler::PdfOcrWorkerScheduler;
