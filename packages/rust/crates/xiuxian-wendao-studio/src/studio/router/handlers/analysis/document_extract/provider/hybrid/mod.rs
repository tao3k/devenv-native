//! Hybrid PDF OCR routing facade.

#[path = "precision_gate/mod.rs"]
mod precision_gate;
mod profile;
mod render;
mod route;
mod structure;
mod types;

#[cfg(test)]
pub(super) use types::{
    DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV, DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI_ENV,
    DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV, DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV, HybridDocumentResourceBatch,
};

#[cfg(test)]
pub(super) use precision_gate::{
    validate_hybrid_page_coverage, validate_hybrid_precision_gate, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
};
#[cfg(test)]
pub(super) use profile::{
    DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV, HybridPdfOcrProfilePlanner,
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles,
    apply_hybrid_page_ocr_profile_plan_for_profiles,
    apply_hybrid_page_ocr2_profile_plan_for_profiles, hybrid_page_ocr_profile_planner_with_lookup,
};
#[cfg(test)]
pub(super) use render::{
    HybridPdfOcr2RegionPlanner, automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, hybrid_page_ocr_input_arrow_path,
    hybrid_page_ocr_region_context_ratio_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_profile_with_lookup, hybrid_page_ocr_render_selection_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup,
};
#[cfg(test)]
pub(super) use route::{has_ocr2_recovery_page_candidates, merge_ocr2_recovery_page_inputs};
#[cfg(test)]
pub(super) use structure::{
    hybrid_document_structure_blocks, write_hybrid_document_resource_artifacts,
};
