use std::collections::BTreeSet;
use std::path::PathBuf;

#[cfg(feature = "document-extract-pdf-source-range")]
use xiuxian_wendao_attachments::pdf::ocr::merge_ocr2_recovery_region_inputs;

#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    PdfRenderRoutingDecision, PdfRenderStatus, hybrid_page_ocr_input_arrow_path,
    merge_ocr2_recovery_page_inputs, sample_hybrid_page_ocr_report, sample_ocr_input,
};

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_input_arrow_path_accepts_complete_render() -> Result<(), String> {
    let report = sample_hybrid_page_ocr_report(
        PdfRenderStatus::Rendered,
        PdfRenderRoutingDecision::HybridPageOcrCandidate,
        2,
        2,
        Some("/tmp/out/_ocr_input.arrow"),
    );

    let path = hybrid_page_ocr_input_arrow_path(&report)?;

    assert_eq!(path, PathBuf::from("/tmp/out/_ocr_input.arrow"));
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_input_arrow_path_accepts_partial_page_render() -> Result<(), String> {
    let report = sample_hybrid_page_ocr_report(
        PdfRenderStatus::Rendered,
        PdfRenderRoutingDecision::HybridPageOcrCandidate,
        3,
        1,
        Some("/tmp/out/_ocr_input.arrow"),
    );

    let path = hybrid_page_ocr_input_arrow_path(&report)?;

    assert_eq!(path, PathBuf::from("/tmp/out/_ocr_input.arrow"));
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_input_arrow_path_rejects_fallback_report() {
    let mut report = sample_hybrid_page_ocr_report(
        PdfRenderStatus::Fallback,
        PdfRenderRoutingDecision::FullDoclingFallback,
        1,
        0,
        None,
    );
    report.error_message = Some("bind Pdfium library: missing".to_string());

    let Err(error) = hybrid_page_ocr_input_arrow_path(&report) else {
        panic!("fallback report should not become OCR input");
    };

    assert!(error.contains("not eligible for hybrid OCR"));
    assert!(error.contains("bind Pdfium library: missing"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn ocr2_recovery_merge_keeps_profile_and_uses_rendered_image() -> Result<(), String> {
    let fast_input = sample_ocr_input(0, "page");
    let mut ocr2_input = sample_ocr_input(1, "page");
    ocr2_input.image_path = "/tmp/out/source-range-placeholder".to_string();
    ocr2_input.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    ocr2_input.ocr_engine = "deepseek-ocr2-direct-vlm".to_string();

    let mut rendered_input = sample_ocr_input(1, "page");
    rendered_input.image_path = "/tmp/out/_ocr2-page-renders/page-00001.png".to_string();
    rendered_input.raster_width_px = 2480;
    rendered_input.raster_height_px = 3508;
    rendered_input.render_dpi = 300;

    let merged = merge_ocr2_recovery_page_inputs(
        vec![fast_input.clone(), ocr2_input],
        vec![rendered_input],
    )?;

    assert_eq!(merged[0].image_path, fast_input.image_path);
    assert_eq!(merged[1].ocr_profile, "deepseek-ocr2-direct-vlm");
    assert_eq!(merged[1].ocr_engine, "deepseek-ocr2-direct-vlm");
    assert_eq!(
        merged[1].image_path,
        "/tmp/out/_ocr2-page-renders/page-00001.png"
    );
    assert_eq!(merged[1].render_dpi, 300);
    assert_eq!(merged[1].raster_width_px, 2480);
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn ocr2_region_merge_deescalates_parent_page_and_appends_region() -> Result<(), String> {
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    ocr2_page.ocr_engine = "deepseek-ocr2-direct-vlm".to_string();

    let mut rendered_region = sample_ocr_input(1, "region");
    rendered_region.image_path =
        "/tmp/out/_ocr2-region-renders/page-00001-region-00001.png".to_string();
    rendered_region.parent_shard_element_id = "render-profile-parent-page".to_string();

    let merged = merge_ocr2_recovery_region_inputs(
        vec![sample_ocr_input(0, "page"), ocr2_page],
        vec![rendered_region],
        &BTreeSet::from([1]),
    )?;

    assert_eq!(merged.len(), 3);
    assert_eq!(merged[1].ocr_profile, "docling-fast-text-ocr");
    assert_eq!(merged[1].ocr_engine, "docling-fast-text-ocr");
    assert_eq!(merged[2].shard_type, "region");
    assert_eq!(merged[2].ocr_profile, "deepseek-ocr2-direct-vlm");
    assert_eq!(merged[2].ocr_engine, "deepseek-ocr2-direct-vlm");
    assert_eq!(
        merged[2].parent_shard_element_id,
        merged[1].shard_element_id
    );
    Ok(())
}
