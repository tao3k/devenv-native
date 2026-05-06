//! Adaptive OCR scheduler facade.

mod core;
mod dispatch;
mod limit;

pub(crate) use core::PdfOcrWorkerScheduler;
pub(super) use limit::DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV;

#[cfg(test)]
pub(super) use dispatch::{endpoint_index_for_request, scheduler_shard_groups};

#[cfg(test)]
pub(super) use limit::{
    DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV, pdf_ocr_worker_limit_with_lookup,
    rendered_region_shard_chunks, rendered_region_shard_chunks_with_composite_size,
    source_pdf_page_range_chunks, source_pdf_page_range_chunks_with_weights,
};

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/scheduler.rs"]
mod tests;
