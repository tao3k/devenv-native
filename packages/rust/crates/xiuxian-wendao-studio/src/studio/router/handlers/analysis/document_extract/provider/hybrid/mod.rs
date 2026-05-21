//! Hybrid PDF OCR routing facade.

#[path = "precision_gate/mod.rs"]
mod precision_gate;
mod profile;
mod render;
mod route;
mod structure;
mod types;

pub(super) use precision_gate::{
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results_for_inputs,
};
pub(super) use profile::{
    HybridPdfOcrProfilePlanner, apply_hybrid_page_ocr_profile_plan,
    hybrid_page_ocr_profile_planner, hybrid_page_ocr_profile_planner_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
pub(super) use render::hybrid_page_ocr_render_profile_with_lookup;
pub(super) use render::{HybridPdfOcr2RegionPlanner, hybrid_page_ocr2_region_planner_with_lookup};
#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(super) use render::{
    automatic_ocr2_recovery_region_requests_for_source_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
};
pub(super) use render::{
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_request_paths, render_hybrid_page_ocr_shards,
};
pub(super) use structure::write_hybrid_document_resource_artifacts;
pub(super) use types::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, HybridDocumentResourceBatch,
    PageRangeDoclingFallbackChunkTiming, PageRangeDoclingFallbackPlanRange,
    PageRangeDoclingFallbackPlanSummary, PageRangeDoclingFallbackSourceProfileSummary,
};
#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(super) use types::{
    HybridPdfOcr2RegionPipelineMode, HybridPdfOcr2RegionRenderChunkMode, HybridPdfOcr2ScaffoldMode,
    hybrid_page_ocr2_region_pipeline_mode_with_lookup,
    hybrid_page_ocr2_region_render_chunk_mode_with_lookup,
    hybrid_page_ocr2_scaffold_mode_with_lookup,
};

#[cfg(test)]
pub(super) use types::{
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV, DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
};

#[cfg(test)]
pub(super) use precision_gate::validate_successful_ocr_results_for_inputs_with_lookup;
#[cfg(test)]
pub(super) use precision_gate::{validate_hybrid_precision_gate, validate_successful_ocr_results};
#[cfg(test)]
pub(super) use profile::{
    DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV, DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV,
    HybridPdfBackendTextTopup,
    apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup,
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles,
    apply_hybrid_page_ocr_profile_plan_for_profiles, hybrid_pdf_backend_text_topup_with_lookup,
};
#[cfg(test)]
pub(super) use render::{
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup,
    hybrid_page_ocr_region_context_ratio_with_lookup, hybrid_page_ocr_render_selection_with_lookup,
};
#[cfg(test)]
pub(super) use route::{has_ocr2_recovery_page_candidates, merge_ocr2_recovery_page_inputs};
#[cfg(test)]
pub(super) use structure::hybrid_document_structure_blocks;
