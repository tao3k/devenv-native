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
    assert_eq!(manifest.geometry.rotation_degrees, 90);
    assert_eq!(manifest.geometry.render_dpi, 300);
    assert!(manifest.geometry.point_to_pixel_scale_x > 4.0);
    assert!(manifest.geometry.point_to_pixel_scale_y > 4.0);
    assert_eq!(manifest.image_mime_type, "image/png");
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
