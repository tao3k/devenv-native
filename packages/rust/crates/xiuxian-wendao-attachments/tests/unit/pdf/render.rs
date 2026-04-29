use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "expected {actual} to be close to {expected}"
    );
}

fn int32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("`{name}` column is not Int32"))
}

fn float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("`{name}` column is not Float64"))
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
}

fn sample_manifest(rotation_degrees: u16) -> PdfPageShardManifest {
    let profile = PdfPageRenderProfile::ocr_default();
    build_shard_manifest(PdfPageShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 2,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        rotation_degrees,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00002.png"),
            sha256: "rasterhash".to_string(),
            width_px: 2400,
            height_px: 3100,
        },
    })
}

fn sample_region_manifest() -> Result<PdfPageShardManifest, String> {
    let page_manifest = sample_manifest(0);
    let profile = PdfPageRenderProfile::ocr_default();
    build_region_shard_manifest(PdfPageRegionShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 2,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        page_crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        region: PdfPageRegion::new(
            7,
            PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
            page_manifest.element_id,
            "000002.000007",
        ),
        rotation_degrees: 0,
        page_raster_width_px: 2400,
        page_raster_height_px: 3100,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00002-region-00007.png"),
            sha256: "regionhash".to_string(),
            width_px: 600,
            height_px: 775,
        },
    })
}

#[test]
fn document_extract_pdf_render_dimensions_follow_dpi_and_rotation() {
    let profile = PdfPageRenderProfile::ocr_default();
    let page_box = PdfPageBox::new(0.0, 0.0, 612.0, 792.0);

    assert_eq!(
        render_dimensions_for_box(page_box, 0, &profile),
        (2550, 3300)
    );
    assert_eq!(
        render_dimensions_for_box(page_box, 90, &profile),
        (3300, 2550)
    );
    assert_eq!(
        render_dimensions_for_box(page_box, 270, &profile),
        (3300, 2550)
    );
}

#[test]
fn document_extract_pdf_render_manifest_preserves_boxes_and_transform() {
    let manifest = sample_manifest(90);

    assert_close(manifest.geometry.media_box.width_points(), 612.0);
    assert_close(manifest.geometry.crop_box.left, 18.0);
    assert_eq!(manifest.shard_type, PdfOcrShardType::Page);
    assert_eq!(manifest.region_index, 0);
    assert_eq!(manifest.reading_order_key, "000002.000000");
    assert_eq!(manifest.source_page_pixel_box.width_px(), 2400);
    assert_eq!(manifest.source_page_pixel_box.height_px(), 3100);
    assert_eq!(manifest.geometry.rotation_degrees, 90);
    assert_eq!(manifest.geometry.render_dpi, 300);
    assert!(manifest.geometry.point_to_pixel_scale_x > 4.0);
    assert!(manifest.geometry.point_to_pixel_scale_y > 4.0);
    assert_eq!(manifest.image_mime_type, "image/png");
}

#[test]
fn document_extract_pdf_render_maps_region_to_source_page_pixels() -> Result<(), String> {
    let pixel_box = region_pixel_box_for_crop(
        PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
        2400,
        3100,
    )?;

    assert_eq!(pixel_box, PdfPagePixelBox::new(600, 1550, 1200, 2325));
    Ok(())
}

#[test]
fn document_extract_pdf_render_region_manifest_preserves_provenance() -> Result<(), String> {
    let page_manifest = sample_manifest(0);
    let region_manifest = sample_region_manifest()?;

    assert_eq!(region_manifest.shard_type, PdfOcrShardType::Region);
    assert_eq!(region_manifest.region_index, 7);
    assert_eq!(
        region_manifest.parent_shard_element_id,
        page_manifest.element_id
    );
    assert_eq!(region_manifest.reading_order_key, "000002.000007");
    assert_eq!(
        region_manifest.source_page_pixel_box,
        PdfPagePixelBox::new(600, 1550, 1200, 2325)
    );
    assert_close(region_manifest.geometry.crop_box.left, 162.0);
    assert_ne!(region_manifest.element_id, page_manifest.element_id);
    Ok(())
}

#[test]
fn document_extract_pdf_render_shard_id_is_content_addressed() {
    let first = sample_manifest(0);
    let second = sample_manifest(0);
    let mut different = sample_manifest(0);
    different.page_index = 3;
    different.element_id = shard_element_id(
        different.source_content_hash.as_str(),
        different.page_index,
        different.render_profile.as_str(),
    );

    assert_eq!(first.element_id, second.element_id);
    assert_ne!(first.element_id, different.element_id);
}

#[test]
fn document_extract_pdf_render_builds_typed_manifest_arrow_batch() -> Result<(), String> {
    let batch = build_shard_manifest_batch(&[sample_manifest(180)])?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().field(0).name(), "sourcePath");
    assert_eq!(batch.schema().field(1).name(), "sourceContentHash");
    assert_eq!(batch.schema().field(11).name(), "mediaLeft");
    assert_eq!(int32_column(&batch, "rotationDegrees")?.value(0), 180);
    assert_close(float64_column(&batch, "cropLeft")?.value(0), 18.0);
    assert_eq!(string_column(&batch, "shardType")?.value(0), "page");
    assert_eq!(int32_column(&batch, "regionIndex")?.value(0), 0);
    assert_eq!(
        string_column(&batch, "readingOrderKey")?.value(0),
        "000002.000000"
    );
    assert_eq!(int32_column(&batch, "sourcePagePixelLeft")?.value(0), 0);
    Ok(())
}

#[test]
fn document_extract_pdf_render_builds_region_manifest_arrow_batch() -> Result<(), String> {
    let batch = build_shard_manifest_batch(&[sample_region_manifest()?])?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(string_column(&batch, "shardType")?.value(0), "region");
    assert_eq!(int32_column(&batch, "regionIndex")?.value(0), 7);
    assert_eq!(
        string_column(&batch, "readingOrderKey")?.value(0),
        "000002.000007"
    );
    assert_eq!(
        int32_column(&batch, "sourcePagePixelBottom")?.value(0),
        2325
    );
    Ok(())
}

#[test]
fn document_extract_pdf_render_builds_ocr_pending_resource_rows() -> Result<(), String> {
    let batch = build_ocr_pending_resource_batch(&[sample_manifest(0)])?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        string_column(&batch, "resourceType")?.value(0),
        "ocr_pending"
    );
    assert_eq!(string_column(&batch, "status")?.value(0), "pending");
    assert!(
        string_column(&batch, "content")?
            .value(0)
            .contains("_ocr_shards.arrow")
    );
    assert!(
        string_column(&batch, "content")?
            .value(0)
            .contains("shard_type=page")
    );
    Ok(())
}

#[test]
fn document_extract_pdf_render_selects_only_ocr_pages_for_shard_fallback() {
    assert_eq!(
        raster_ocr_page_indices(9, &[1, 3, 3, 12], false),
        vec![0, 2]
    );
}

#[test]
fn document_extract_pdf_render_selects_all_pages_for_scanned_without_hints() {
    assert_eq!(raster_ocr_page_indices(3, &[], true), vec![0, 1, 2]);
}

#[test]
fn document_extract_pdf_render_escalates_mixed_without_hints_to_page_ocr() {
    assert!(should_render_all_when_no_ocr_hints(&routing_signals(
        PdfInspectorPdfType::Mixed,
        false,
    )));
    assert!(should_render_all_when_no_ocr_hints(&routing_signals(
        PdfInspectorPdfType::Scanned,
        false,
    )));
    assert!(should_render_all_when_no_ocr_hints(&routing_signals(
        PdfInspectorPdfType::TextBased,
        true,
    )));
    assert!(!should_render_all_when_no_ocr_hints(&routing_signals(
        PdfInspectorPdfType::TextBased,
        false,
    )));
}

fn routing_signals(pdf_type: PdfInspectorPdfType, is_complex: bool) -> PdfInspectorRoutingSignals {
    PdfInspectorRoutingSignals {
        pdf_type,
        page_count: 3,
        confidence: 0.95,
        pages_needing_ocr: Vec::new(),
        is_complex,
        has_encoding_issues: false,
    }
}
