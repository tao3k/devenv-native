#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV, DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI_ENV, DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
    HybridPdfOcr2RegionPlanner, HybridPdfOcrProfilePlanner, PdfPageRenderSelection,
    PdfRenderRoutingDecision, PdfRenderStatus, apply_hybrid_page_ocr_profile_plan_for_profiles,
    apply_hybrid_page_ocr2_profile_plan_for_profiles, assert_close,
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, has_ocr2_recovery_page_candidates,
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_profile_planner_with_lookup,
    hybrid_page_ocr_region_context_ratio_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_profile_with_lookup, hybrid_page_ocr_render_selection_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup, merge_ocr2_recovery_page_inputs,
    sample_hybrid_page_ocr_report, sample_ocr_input, sample_ocr_result,
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
};

mod coverage;
mod input_arrow;
mod regions;
mod render_profile;
mod scope;
