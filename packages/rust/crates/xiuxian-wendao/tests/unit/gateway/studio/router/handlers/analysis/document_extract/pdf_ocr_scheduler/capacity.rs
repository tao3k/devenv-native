use super::*;
use xiuxian_wendao_attachments::pdf::ocr::{PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput};

#[test]
fn adaptive_capacity_increases_after_healthy_window() {
    let controller = OcrCapacityController::new_with_current_budget(6, 2);

    controller.record_success(1, 100);
    assert_eq!(controller.snapshot().current_worker_budget, 2);

    controller.record_success(1, 100);
    let snapshot = controller.snapshot();
    assert_eq!(snapshot.current_worker_budget, 3);
    assert_eq!(snapshot.budget_increase_events, 1);
}

#[test]
fn adaptive_capacity_halves_on_failure() {
    let controller = OcrCapacityController::new_with_current_budget(8, 7);

    controller.record_failure();
    let snapshot = controller.snapshot();

    assert_eq!(snapshot.current_worker_budget, 4);
    assert_eq!(snapshot.budget_decrease_events, 1);
}

#[test]
fn adaptive_capacity_halves_on_latency_pressure() {
    let controller = OcrCapacityController::new_with_current_budget(8, 6);

    controller.record_success(1, PRESSURE_LATENCY_MS + 1);

    assert_eq!(controller.snapshot().current_worker_budget, 3);
}

#[test]
fn source_range_budget_is_sublinear_under_current_budget() {
    let controller = OcrCapacityController::new_with_current_budget(12, 12);

    let budget = controller.budget_for_lane(21, OcrSchedulerLane::SourcePdfPageRange, None);

    assert_eq!(budget, 4);
}

#[test]
fn source_range_override_is_capped_by_current_budget_and_shards() {
    let controller = OcrCapacityController::new_with_current_budget(12, 5);

    let budget = controller.budget_for_lane(3, OcrSchedulerLane::SourcePdfPageRange, Some(99));

    assert_eq!(budget, 3);
}

#[test]
fn rendered_region_uses_current_budget() {
    let controller = OcrCapacityController::new_with_current_budget(12, 5);

    let budget = controller.budget_for_lane(21, OcrSchedulerLane::RenderedRegion, None);

    assert_eq!(budget, 5);
}

#[test]
fn contiguous_source_pdf_page_range_detects_batchable_inputs() {
    let inputs = vec![
        sample_ocr_input("/tmp/source.pdf", 0, "page"),
        sample_ocr_input("/tmp/source.pdf", 1, "page"),
        sample_ocr_input("/tmp/source.pdf", 2, "page"),
    ];

    assert!(is_contiguous_source_pdf_page_range(inputs.as_slice()));
}

#[test]
fn contiguous_source_pdf_page_range_rejects_regions_and_gaps() {
    let region_inputs = vec![
        sample_ocr_input("/tmp/source.pdf", 0, "page"),
        sample_ocr_input("/tmp/source.pdf", 1, "region"),
    ];
    let gap_inputs = vec![
        sample_ocr_input("/tmp/source.pdf", 0, "page"),
        sample_ocr_input("/tmp/source.pdf", 2, "page"),
    ];

    assert!(!is_contiguous_source_pdf_page_range(
        region_inputs.as_slice()
    ));
    assert!(!is_contiguous_source_pdf_page_range(gap_inputs.as_slice()));
}

fn sample_ocr_input(source_path: &str, page_index: u32, shard_type: &str) -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: source_path.to_string(),
        source_content_hash: "hash".to_string(),
        page_index,
        image_path: format!("/tmp/page-{page_index}.png"),
        image_mime_type: "image/png".to_string(),
        raster_sha256: format!("raster-{page_index}"),
        render_profile: "source-pdf-page-range-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        ocr_engine: "docling".to_string(),
        preferred_languages: vec!["en".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 0,
        raster_height_px: 0,
        render_dpi: 0,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 100.0,
        crop_top: 100.0,
        point_to_pixel_scale_x: 1.0,
        point_to_pixel_scale_y: 1.0,
        shard_element_id: format!("{shard_type}-{page_index}"),
        shard_type: shard_type.to_string(),
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: format!("{page_index:06}.000000"),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 0,
        source_page_pixel_bottom: 0,
    }
}
