//! Cache materialization for Episteme source-contract extraction runs.
//!
//! The user-owned Episteme repository owns source contracts and ledgers.
//! This crate owns deterministic cache materialization so external repositories
//! do not need local runtime scripts.

mod docling_document;
mod image_ocr;
mod legacy_office;
mod materialization_support;
mod path;
mod task;

pub use docling_document::{
    EPISTEME_DOCLING_DOCUMENT_RESULTS_JSONL, EPISTEME_DOCLING_DOCUMENT_ROUTE,
    EPISTEME_DOCLING_DOCUMENT_WRAPPER_SCHEMA, EpistemeDoclingDocumentCacheBridgeReport,
    read_docling_document_tasks_tsv, skipped_docling_document_cache_bridge_report,
    validate_docling_document_tasks, write_docling_document_cache_outputs,
};
pub use image_ocr::{
    EPISTEME_IMAGE_OCR_RESULTS_JSONL, EPISTEME_IMAGE_OCR_ROUTE, EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA,
    EpistemeImageOcrCacheBridgeReport, read_image_ocr_tasks_tsv,
    skipped_image_ocr_cache_bridge_report, validate_image_ocr_tasks, write_image_ocr_cache_outputs,
};
pub use legacy_office::{
    EPISTEME_LEGACY_OFFICE_CONVERSION_RECEIPT_JSON,
    EPISTEME_LEGACY_OFFICE_CONVERSION_WRAPPER_SCHEMA, EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE,
    EpistemeLegacyOfficeConversionReport, EpistemeLegacyOfficeConversionRequest,
    convert_legacy_office_tasks, read_legacy_office_conversion_tasks_tsv,
    validate_legacy_office_conversion_tasks,
};
pub use task::{EpistemeCacheTask, EpistemeCacheTaskCategory, EpistemeCacheTaskStatus};
