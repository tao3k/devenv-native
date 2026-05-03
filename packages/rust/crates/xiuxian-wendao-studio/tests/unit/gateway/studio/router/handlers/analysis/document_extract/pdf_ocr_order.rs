use super::{PdfOcrShardInput, order_ocr_results_by_inputs, validate_ocr_result_matches_input};
use xiuxian_wendao_attachments::pdf::ocr::{PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardResult};

#[test]
fn restores_ocr_results_to_input_order() -> Result<(), String> {
    let inputs = vec![
        sample_ocr_input(0, "page"),
        sample_ocr_input(1, "page"),
        sample_ocr_input(2, "page"),
    ];
    let results = vec![
        PdfOcrShardResult::succeeded(&inputs[2], "page 2", 1.0),
        PdfOcrShardResult::succeeded(&inputs[0], "page 0", 1.0),
        PdfOcrShardResult::succeeded(&inputs[1], "page 1", 1.0),
    ];

    let ordered = order_ocr_results_by_inputs(inputs.as_slice(), results)?;

    assert_eq!(ordered[0].shard_element_id, "page-shard-0");
    assert_eq!(ordered[1].shard_element_id, "page-shard-1");
    assert_eq!(ordered[2].shard_element_id, "page-shard-2");
    Ok(())
}

#[test]
fn rejects_duplicate_ocr_result_shards() {
    let inputs = vec![sample_ocr_input(0, "page"), sample_ocr_input(1, "page")];
    let duplicate = PdfOcrShardResult::succeeded(&inputs[0], "page", 1.0);
    let Err(error) =
        order_ocr_results_by_inputs(inputs.as_slice(), vec![duplicate.clone(), duplicate])
    else {
        panic!("expected duplicate result to fail");
    };

    assert!(error.contains("duplicate OCR shard result id"));
}

#[test]
fn rejects_mismatched_ocr_result_hashes() {
    let input = sample_ocr_input(0, "page");
    let mut result = PdfOcrShardResult::succeeded(&input, "page", 1.0);
    result.raster_sha256 = "different".to_string();

    let Err(error) = validate_ocr_result_matches_input(&input, &result) else {
        panic!("expected mismatched raster hash to fail");
    };

    assert!(error.contains("raster hash"));
}

fn sample_ocr_input(page_index: u32, shard_type: &str) -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index,
        image_path: format!("/tmp/page-{page_index:05}.png"),
        image_mime_type: "image/png".to_string(),
        raster_sha256: format!("rasterhash-{page_index}"),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        ocr_engine: "docling-compatible-ocr".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 2400,
        raster_height_px: 3100,
        render_dpi: 300,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 612.0,
        crop_top: 792.0,
        point_to_pixel_scale_x: 3.921_568_627,
        point_to_pixel_scale_y: 3.914_141_414,
        shard_element_id: format!("{shard_type}-shard-{page_index}"),
        shard_type: shard_type.to_string(),
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: format!("{page_index:06}.000000"),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 2400,
        source_page_pixel_bottom: 3100,
    }
}
