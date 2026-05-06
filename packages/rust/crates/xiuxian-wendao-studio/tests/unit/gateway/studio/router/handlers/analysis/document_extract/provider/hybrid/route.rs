use std::path::Path;

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput,
};
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile,
};

use super::{
    OCR2_REGION_SCAFFOLD_FILE_NAME, cached_ocr2_region_render_report,
    has_ocr2_recovery_page_candidates, ocr2_region_render_cache_key, ocr2_region_scaffold_payload,
    write_ocr2_region_scaffold_sidecar_with_lookup,
};
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::DOCUMENT_EXTRACT_PDF_OCR2_SCAFFOLD_MODE_ENV;

#[test]
fn ocr2_region_scaffold_payload_is_disabled_by_default() -> Result<(), String> {
    let region = sample_region_input();

    let payload =
        ocr2_region_scaffold_payload(Path::new("/tmp/source.pdf"), &[region], false, &|_key| None)?;

    assert!(payload.is_none());
    Ok(())
}

#[test]
fn ocr2_region_scaffold_payload_records_region_fingerprints() -> Result<(), String> {
    let mut region = sample_region_input();
    region.parent_shard_element_id = "parent-page-shard".to_string();
    region.source_content_hash = "parent-page-hash".to_string();
    region.raster_sha256 = "region-raster-hash".to_string();
    region.render_dpi = 300;

    let payload = ocr2_region_scaffold_payload(
        Path::new("/tmp/source.pdf"),
        &[region],
        true,
        &scaffold_enabled_lookup,
    )?
    .ok_or_else(|| "expected OCR2 scaffold payload".to_string())?;
    let items = payload
        .get("items")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "missing scaffold items".to_string())?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["scaffoldKind"], "manual_region_candidate");
    assert_eq!(items[0]["parentShardElementId"], "parent-page-shard");
    assert_eq!(items[0]["sourceContentHash"], "parent-page-hash");
    assert_eq!(items[0]["rasterSha256"], "region-raster-hash");
    assert_eq!(items[0]["renderDpi"], 300);
    assert_eq!(items[0]["sourcePagePixelBox"]["right"], 1000);
    Ok(())
}

#[test]
fn ocr2_region_scaffold_sidecar_writes_only_when_enabled() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let region = sample_region_input();

    write_ocr2_region_scaffold_sidecar_with_lookup(
        Path::new("/tmp/source.pdf"),
        temp.path(),
        std::slice::from_ref(&region),
        false,
        &|_key| None,
    )?;
    assert!(!temp.path().join(OCR2_REGION_SCAFFOLD_FILE_NAME).exists());

    write_ocr2_region_scaffold_sidecar_with_lookup(
        Path::new("/tmp/source.pdf"),
        temp.path(),
        std::slice::from_ref(&region),
        false,
        &scaffold_enabled_lookup,
    )?;
    assert!(temp.path().join(OCR2_REGION_SCAFFOLD_FILE_NAME).is_file());
    Ok(())
}

#[test]
fn ocr2_region_candidate_detection_requires_direct_page_profile() {
    let mut input = sample_region_input();
    assert!(!has_ocr2_recovery_page_candidates(&[input.clone()]));

    input.shard_type = "page".to_string();
    input.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    assert!(!has_ocr2_recovery_page_candidates(&[input.clone()]));

    input.ocr_profile = PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE.to_string();
    assert!(has_ocr2_recovery_page_candidates(&[input]));
}

#[test]
fn ocr2_region_render_cache_key_tracks_source_profile_and_region() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(source.as_path(), b"source-a").map_err(|error| error.to_string())?;
    let profile = PdfPageRenderProfile::ocr_default();
    let region = sample_region_request(1);

    let baseline =
        ocr2_region_render_cache_key(source.as_path(), &profile, std::slice::from_ref(&region))?;
    assert_eq!(
        baseline,
        ocr2_region_render_cache_key(source.as_path(), &profile, std::slice::from_ref(&region),)?
    );

    let mut changed_region = region.clone();
    changed_region.region_box = PdfPageBox::new(10.0, 10.0, 220.0, 260.0);
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key(source.as_path(), &profile, &[changed_region])?
    );

    let mut changed_profile = profile.clone();
    changed_profile.dpi = 360;
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key(
            source.as_path(),
            &changed_profile,
            std::slice::from_ref(&region),
        )?
    );

    std::fs::write(source.as_path(), b"source-b").map_err(|error| error.to_string())?;
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key(source.as_path(), &profile, &[region])?
    );
    Ok(())
}

#[test]
fn cached_ocr2_region_render_report_rejects_missing_artifacts() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(source.as_path(), b"source").map_err(|error| error.to_string())?;

    let cached = cached_ocr2_region_render_report(
        source.as_path(),
        temp.path().join("missing").as_path(),
        1,
        &PdfPageRenderProfile::ocr_default(),
        1,
    )?;

    assert!(cached.is_none());
    Ok(())
}

fn scaffold_enabled_lookup(key: &str) -> Option<String> {
    (key == DOCUMENT_EXTRACT_PDF_OCR2_SCAFFOLD_MODE_ENV).then(|| "region-table-json".to_string())
}

fn sample_region_request(region_index: u32) -> PdfPageRegionRenderRequest {
    PdfPageRegionRenderRequest::new(
        1,
        region_index,
        PdfPageBox::new(10.0, 20.0, 110.0, 220.0),
        Some(format!("000001.{region_index:06}")),
    )
}

fn sample_region_input() -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index: 1,
        image_path: "/tmp/out/_ocr2-region-renders/page-00001-region-00001.png".to_string(),
        image_mime_type: "image/png".to_string(),
        raster_sha256: "raster-1".to_string(),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE.to_string(),
        ocr_engine: "deepseek-ocr2-direct-vlm".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 1000,
        raster_height_px: 1000,
        render_dpi: 300,
        rotation_degrees: 0,
        crop_left: 10.0,
        crop_bottom: 20.0,
        crop_right: 110.0,
        crop_top: 220.0,
        point_to_pixel_scale_x: 3.0,
        point_to_pixel_scale_y: 3.0,
        shard_element_id: "region-shard".to_string(),
        shard_type: "region".to_string(),
        region_index: 1,
        parent_shard_element_id: "parent-shard".to_string(),
        reading_order_key: "000001.000050".to_string(),
        source_page_pixel_left: 0,
        source_page_pixel_top: 100,
        source_page_pixel_right: 1000,
        source_page_pixel_bottom: 900,
    }
}
