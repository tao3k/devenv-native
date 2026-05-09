#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV, DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV,
    DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV, DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
    HybridPdfBackendTextTopup, HybridPdfOcrProfilePlanner, PdfPageRenderSelection,
    apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles,
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup,
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles,
    apply_hybrid_page_ocr_profile_plan_for_profiles, hybrid_page_ocr_profile_planner_with_lookup,
    hybrid_page_ocr_render_profile_with_lookup, hybrid_page_ocr_render_selection_with_lookup,
    hybrid_pdf_backend_text_topup_with_lookup, sample_ocr_input,
};

include!("selection.rs");
include!("plans.rs");

#[cfg(feature = "document-extract-pdf-source-range")]
fn sample_source_page_profile(
    page_index: u32,
    fast_profile_risk: bool,
) -> xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
    xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
        page_index,
        content_bytes: 1024,
        operation_count: if fast_profile_risk { 695 } else { 24 },
        text_show_ops: if fast_profile_risk { 195 } else { 10 },
        path_ops: if fast_profile_risk { 73 } else { 8 },
        rectangle_ops: if fast_profile_risk { 2 } else { 0 },
        draw_object_ops: 0,
        estimated_weight: 1,
    }
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn sample_source_page_profile_with_counts(
    page_index: u32,
    text_show_ops: u32,
    path_ops: u32,
    rectangle_ops: u32,
    draw_object_ops: u32,
    operation_count: u32,
    content_bytes: u32,
) -> xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
    xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
        page_index,
        content_bytes,
        operation_count,
        text_show_ops,
        path_ops,
        rectangle_ops,
        draw_object_ops,
        estimated_weight: 1,
    }
}
