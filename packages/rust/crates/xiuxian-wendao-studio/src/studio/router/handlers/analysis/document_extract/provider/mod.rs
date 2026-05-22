//! Flight provider facade for document extraction routing.

#[cfg(feature = "document-extract-audio-shards")]
mod audio;
mod core;
#[cfg(feature = "document-extract-pdf-source-range")]
#[path = "hybrid/mod.rs"]
mod hybrid;
mod jobs;
mod native_org;
mod route;
mod runtime;
mod transport;

use core::{
    DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, DOCUMENT_EXTRACT_ENDPOINT_ENV,
    DOCUMENT_EXTRACT_ENDPOINTS_ENV, DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES,
    DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV, DOCUMENT_EXTRACT_PROVIDER_RUNTIMES,
    DocumentExtractProviderRuntime,
};
pub(crate) use core::{DocumentExtractRuntimeSnapshot, StudioDocumentExtractFlightRouteProvider};

#[cfg(test)]
use arrow::record_batch::RecordBatch as EngineRecordBatch;
#[cfg(test)]
use std::sync::Arc;
#[cfg(all(test, feature = "document-extract-pdf-source-range"))]
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus,
};
#[cfg(all(
    test,
    feature = "document-extract-pdf-source-range",
    feature = "document-extract-pdf-render"
))]
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRenderSelection, PdfPageRenderShardReport, PdfRenderRoutingDecision, PdfRenderStatus,
};
#[cfg(all(test, feature = "document-extract-pdf-source-range"))]
use xiuxian_wendao_attachments::pdf::structure::build_document_structure_batch;

#[cfg(test)]
use super::arrow_cache::{read_arrow_file, write_arrow_file};
#[cfg(test)]
use super::registry::DocumentExtractJobRegistry;
#[cfg(all(test, feature = "document-extract-pdf-source-range"))]
use hybrid::validate_successful_ocr_results_for_inputs_with_lookup;
#[cfg(all(
    test,
    feature = "document-extract-pdf-source-range",
    feature = "document-extract-pdf-render"
))]
use hybrid::{
    DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV, DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV, DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV, HybridPdfBackendTextTopup,
    HybridPdfOcr2RegionPlanner, HybridPdfOcrProfilePlanner,
    apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup,
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles,
    apply_hybrid_page_ocr_profile_plan_for_profiles,
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, hybrid_page_ocr_input_arrow_path,
    hybrid_page_ocr_profile_planner_with_lookup, hybrid_page_ocr_region_context_ratio_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_selection_with_lookup, hybrid_page_ocr2_region_patch_sizing_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup, hybrid_pdf_backend_text_topup_with_lookup,
};
#[cfg(all(test, feature = "document-extract-pdf-source-range"))]
use hybrid::{
    HybridDocumentResourceBatch, hybrid_document_structure_blocks, validate_hybrid_page_coverage,
    validate_hybrid_precision_gate, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
    write_hybrid_document_resource_artifacts,
};
#[cfg(all(test, feature = "document-extract-pdf-render"))]
use hybrid::{
    has_ocr2_recovery_page_candidates, hybrid_page_ocr_render_profile_with_lookup,
    merge_ocr2_recovery_page_inputs,
};
#[cfg(test)]
use jobs::document_extract_batches_are_cacheable;
#[cfg(test)]
use runtime::{
    document_extract_conversion_concurrency_limit_with_lookup,
    shared_document_extract_provider_runtime,
};

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/mod.rs"]
mod tests;
