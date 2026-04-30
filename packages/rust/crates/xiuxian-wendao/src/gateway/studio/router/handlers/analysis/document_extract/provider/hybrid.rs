//! Hybrid PDF OCR routing facade.

mod render;
mod route;
mod structure;
mod types;
mod validate;

#[cfg(test)]
pub(super) use types::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
    HybridDocumentResourceBatch,
};

#[cfg(test)]
pub(super) use render::{
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_selection_with_lookup,
};
#[cfg(test)]
pub(super) use structure::{
    hybrid_document_structure_blocks, write_hybrid_document_resource_artifacts,
};
#[cfg(test)]
pub(super) use validate::{
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
};
