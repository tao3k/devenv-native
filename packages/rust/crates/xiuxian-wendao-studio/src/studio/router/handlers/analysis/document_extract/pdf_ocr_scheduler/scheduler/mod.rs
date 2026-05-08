//! Adaptive OCR scheduler facade.

mod core;
mod dispatch;
mod limit;
mod local_text;

pub(crate) use core::PdfOcrWorkerScheduler;
pub(super) use limit::DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS_ENV;

#[cfg(test)]
pub(super) use dispatch::{
    endpoint_index_for_request, scheduler_shard_groups,
    source_pdf_page_range_chunk_endpoint_index_with_lookup,
    source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup,
};

#[cfg(test)]
pub(super) use local_text::{
    local_backend_and_fast_text_results_for_tests,
    local_backend_text_error_fail_fast_results_for_tests, local_backend_text_results_for_tests,
    local_empty_backend_text_dispatch_python_results_for_tests,
    local_empty_backend_text_fail_fast_results_for_tests,
};

#[cfg(test)]
use dispatch::scheduler_trace_for_chunk;

#[cfg(test)]
use super::capacity::OcrSchedulerLane;

#[cfg(test)]
pub(super) use limit::{
    DOCUMENT_EXTRACT_PDF_OCR_WORKERS_ENV, pdf_ocr_worker_limit_with_lookup,
    rendered_region_shard_chunks, rendered_region_shard_chunks_with_composite_size,
    source_pdf_page_range_chunks, source_pdf_page_range_chunks_with_fast_text_split,
    source_pdf_page_range_chunks_with_weights, source_pdf_page_range_dispatch_budget,
    source_pdf_page_range_dispatch_budget_with_region_pipeline,
    source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split,
    source_pdf_page_range_dispatch_chunks,
};

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/scheduler.rs"]
mod tests;
