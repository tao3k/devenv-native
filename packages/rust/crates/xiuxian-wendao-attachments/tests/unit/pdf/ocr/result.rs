use std::{collections::HashMap, sync::Arc};

use arrow::array::Array;
use arrow::record_batch::RecordBatch;

use super::{
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION, PdfOcrShardResult, PdfOcrWorkerProfile, assert_close,
    build_ocr_result_resource_batch, build_ocr_shard_inputs, build_ocr_shard_result_batch,
    decode_ocr_shard_result_batch, float64_column, sample_manifest, string_column,
};

#[test]
fn document_extract_pdf_ocr_result_batch_preserves_success_and_failure() -> Result<(), String> {
    let inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    let success = PdfOcrShardResult::succeeded(&inputs[0], "recognized text", 0.98);
    let failure = PdfOcrShardResult::failed(&inputs[0], "ocr failed");
    let batch = build_ocr_shard_result_batch(&[success.clone(), failure.clone()])?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(
        string_column(&batch, "contractVersion")?.value(0),
        PDF_OCR_SHARD_RESULT_SCHEMA_VERSION
    );
    assert_eq!(string_column(&batch, "status")?.value(0), "succeeded");
    assert_eq!(string_column(&batch, "text")?.value(0), "recognized text");
    assert_close(float64_column(&batch, "confidence")?.value(0), 0.98);
    assert_eq!(string_column(&batch, "status")?.value(1), "failed");
    assert!(string_column(&batch, "text")?.is_null(1));
    assert_eq!(
        string_column(&batch, "errorMessage")?.value(1),
        "ocr failed"
    );
    assert_eq!(success.element_id, failure.element_id);
    Ok(())
}

#[test]
fn document_extract_pdf_ocr_decodes_result_batch() -> Result<(), String> {
    let inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    let success = PdfOcrShardResult::succeeded(&inputs[0], "recognized text", 0.98);
    let skipped = PdfOcrShardResult::skipped(&inputs[0], "worker not configured");
    let batch = build_ocr_shard_result_batch(&[success.clone(), skipped.clone()])?;

    let decoded = decode_ocr_shard_result_batch(&batch)?;

    assert_eq!(decoded, vec![success, skipped]);
    Ok(())
}

#[test]
fn document_extract_pdf_ocr_decodes_metadata_free_worker_result_batch() -> Result<(), String> {
    let inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    let success = PdfOcrShardResult::succeeded(&inputs[0], "recognized text", 0.98);
    let batch = build_ocr_shard_result_batch(std::slice::from_ref(&success))?;
    let batch = without_schema_metadata(&batch)?;

    let decoded = decode_ocr_shard_result_batch(&batch)?;

    assert_eq!(decoded, vec![success]);
    Ok(())
}

#[test]
fn document_extract_pdf_ocr_result_resource_rows_use_stable_schema() -> Result<(), String> {
    let inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    let success = PdfOcrShardResult::succeeded(&inputs[0], "recognized text", 0.98);
    let skipped = PdfOcrShardResult::skipped(&inputs[0], "below quality gate");
    let batch = build_ocr_result_resource_batch(&[success, skipped])?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().field(0).name(), "sourcePath");
    assert_eq!(batch.schema().field(8).name(), "elementId");
    assert_eq!(string_column(&batch, "resourceType")?.value(0), "ocr_text");
    assert_eq!(
        string_column(&batch, "content")?.value(0),
        "recognized text"
    );
    assert_eq!(string_column(&batch, "mimeType")?.value(0), "text/plain");
    assert_eq!(string_column(&batch, "status")?.value(0), "succeeded");
    assert_eq!(
        string_column(&batch, "resourceType")?.value(1),
        "ocr_skipped"
    );
    assert_eq!(
        string_column(&batch, "content")?.value(1),
        "below quality gate"
    );
    Ok(())
}

#[test]
fn document_extract_pdf_ocr_result_id_changes_with_profile() {
    let manifest = sample_manifest();
    let first_input = build_ocr_shard_inputs(
        std::slice::from_ref(&manifest),
        &PdfOcrWorkerProfile::docling_compatible(),
    )
    .remove(0);
    let second_input = build_ocr_shard_inputs(
        &[manifest],
        &PdfOcrWorkerProfile {
            profile_id: "other-ocr-profile".to_string(),
            ..PdfOcrWorkerProfile::docling_compatible()
        },
    )
    .remove(0);

    let first = PdfOcrShardResult::succeeded(&first_input, "recognized text", 0.98);
    let second = PdfOcrShardResult::succeeded(&second_input, "recognized text", 0.98);

    assert_ne!(first.element_id, second.element_id);
    assert_eq!(first.shard_element_id, second.shard_element_id);
}

fn without_schema_metadata(batch: &RecordBatch) -> Result<RecordBatch, String> {
    let schema = Arc::new(
        batch
            .schema()
            .as_ref()
            .clone()
            .with_metadata(HashMap::new()),
    );
    RecordBatch::try_new(schema, batch.columns().to_vec())
        .map_err(|error| format!("rebuild OCR result batch without metadata: {error}"))
}

#[test]
fn document_extract_pdf_ocr_result_id_changes_with_shard_element_id() {
    let first_input = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile::docling_compatible(),
    )
    .remove(0);
    let mut second_input = first_input.clone();
    second_input.shard_type = "region".to_string();
    second_input.region_index = 1;
    second_input.shard_element_id = "same-raster-other-region".to_string();
    second_input.reading_order_key = "000003.000001".to_string();

    let first = PdfOcrShardResult::succeeded(&first_input, "recognized text", 0.98);
    let second = PdfOcrShardResult::succeeded(&second_input, "recognized text", 0.98);

    assert_eq!(first_input.raster_sha256, second_input.raster_sha256);
    assert_ne!(first.element_id, second.element_id);
}
