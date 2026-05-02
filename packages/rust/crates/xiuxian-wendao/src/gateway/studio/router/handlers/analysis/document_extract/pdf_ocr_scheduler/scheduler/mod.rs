//! Adaptive OCR scheduler facade.

mod core;
mod dispatch;
mod limit;

pub(crate) use core::PdfOcrWorkerScheduler;
pub(super) use limit::DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV;

#[cfg(test)]
pub(super) use dispatch::endpoint_index_for_request;

#[cfg(test)]
pub(super) use limit::{
    DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV, pdf_ocr_worker_limit_with_lookup,
    source_pdf_page_range_chunks,
};

#[cfg(test)]
#[path = "../../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/scheduler.rs"]
mod tests;
