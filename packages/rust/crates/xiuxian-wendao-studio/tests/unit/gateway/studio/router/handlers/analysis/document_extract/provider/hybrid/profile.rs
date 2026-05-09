use super::{
    DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV, HybridPdfOcrProfilePlanner,
    PDF_OCR_DEFAULT_ENGINE, apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles,
    hybrid_page_ocr_profile_planner_with_lookup,
};
use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PdfOcrShardInput,
};
use xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile;

fn input(page_index: u32) -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: "xiuxian_wendao.pdf_ocr_shard_input.v1".to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "hash".to_string(),
        page_index,
        image_path: String::new(),
        image_mime_type: String::new(),
        raster_sha256: String::new(),
        render_profile: "source-pdf-page-range-shards-v1".to_string(),
        ocr_profile: PDF_OCR_DEFAULT_PROFILE.to_string(),
        ocr_engine: PDF_OCR_DEFAULT_ENGINE.to_string(),
        preferred_languages: Vec::new(),
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 0,
        raster_height_px: 0,
        render_dpi: 300,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 0.0,
        crop_top: 0.0,
        point_to_pixel_scale_x: 1.0,
        point_to_pixel_scale_y: 1.0,
        shard_element_id: format!("page-{page_index}"),
        shard_type: "page".to_string(),
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: format!("{page_index:06}"),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 0,
        source_page_pixel_bottom: 0,
    }
}

fn profile(
    page_index: u32,
    content_bytes: u32,
    operation_count: u32,
    text_show_ops: u32,
    path_ops: u32,
    rectangle_ops: u32,
    draw_object_ops: u32,
) -> PdfSourcePageProfile {
    PdfSourcePageProfile {
        page_index,
        content_bytes,
        operation_count,
        text_show_ops,
        path_ops,
        rectangle_ops,
        draw_object_ops,
        estimated_weight: operation_count,
    }
}

#[test]
fn parses_docling_structure_recovery_profile_planner() {
    let planner = hybrid_page_ocr_profile_planner_with_lookup(&|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
            .then(|| "docling-structure-recovery".to_string())
    });

    assert_eq!(
        planner,
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery
    );
    assert!(!planner.requires_rendered_page_images());
}

#[test]
fn docling_structure_recovery_keeps_structure_pages_off_backend_text() {
    let planned = apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles(
        vec![input(0), input(1), input(2), input(3)],
        &[
            profile(0, 8_192, 80, 20, 8, 0, 0),
            profile(1, 8_192, 80, 20, 8, 0, 1),
            profile(2, 8_192, 720, 360, 8, 0, 0),
            profile(3, 8_192, 720, 180, 70, 0, 0),
        ],
    );

    assert_eq!(planned[0].ocr_profile, PDF_OCR_BACKEND_TEXT_PROFILE);
    assert_eq!(planned[1].ocr_profile, PDF_OCR_DEFAULT_PROFILE);
    assert_eq!(planned[2].ocr_profile, PDF_OCR_FAST_TEXT_PROFILE);
    assert_eq!(planned[3].ocr_profile, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE);
}
