use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;

use crate::pdf::metrics::{
    DOCUMENT_METRICS_SCHEMA_VERSION, PdfOcrShardMetric, build_pdf_ocr_metrics_batch,
    document_metrics_schema,
};
use crate::pdf::ocr::{PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput, PdfOcrShardResult};

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
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

fn sample_input() -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index: 2,
        image_path: "/tmp/out/source-page-range-00002.webp".to_string(),
        image_mime_type: "application/pdf".to_string(),
        raster_sha256: "rasterhash".to_string(),
        render_profile: "source-pdf-page-range-v1".to_string(),
        ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
        ocr_engine: "docling-compatible-ocr".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 1836,
        raster_height_px: 2376,
        render_dpi: 216,
        rotation_degrees: 0,
        crop_left: 0.0,
        crop_bottom: 0.0,
        crop_right: 612.0,
        crop_top: 792.0,
        point_to_pixel_scale_x: 3.0,
        point_to_pixel_scale_y: 3.0,
        shard_element_id: "shard-2".to_string(),
        shard_type: "page".to_string(),
        region_index: 0,
        parent_shard_element_id: String::new(),
        reading_order_key: "000002.000000".to_string(),
        source_page_pixel_left: 0,
        source_page_pixel_top: 0,
        source_page_pixel_right: 1836,
        source_page_pixel_bottom: 2376,
    }
}

#[test]
fn pdf_ocr_metrics_batch_uses_stable_schema() -> Result<(), String> {
    let input = sample_input();
    let result = PdfOcrShardResult::succeeded(&input, "recognized text", 0.91);
    let metric = PdfOcrShardMetric::from_ocr_result(&input, &result, 21, Some(1234.5));
    let batch = build_pdf_ocr_metrics_batch(&[metric])?;

    assert_eq!(batch.schema(), document_metrics_schema());
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        string_column(&batch, "contractVersion")?.value(0),
        DOCUMENT_METRICS_SCHEMA_VERSION
    );
    assert_eq!(
        string_column(&batch, "sourcePath")?.value(0),
        "/tmp/source.pdf"
    );
    assert_eq!(int32_column(&batch, "pageIndex")?.value(0), 2);
    assert_eq!(string_column(&batch, "chunkId")?.value(0), "000002.000000");
    assert_eq!(string_column(&batch, "shardElementId")?.value(0), "shard-2");
    assert_eq!(string_column(&batch, "status")?.value(0), "succeeded");
    assert!(float64_column(&batch, "doclingConvertMs")?.is_null(0));
    let scheduler_elapsed_ms = float64_column(&batch, "rustSchedulerElapsedMs")?.value(0);
    assert!((scheduler_elapsed_ms - 1234.5).abs() < f64::EPSILON);
    assert_eq!(int32_column(&batch, "pageCount")?.value(0), 21);
    assert_eq!(int32_column(&batch, "bboxCount")?.value(0), 1);
    assert_eq!(int32_column(&batch, "resultChars")?.value(0), 15);
    assert!(
        string_column(&batch, "provenance")?
            .value(0)
            .contains(r#""source":"rust_hybrid_ocr_scheduler""#)
    );
    Ok(())
}
