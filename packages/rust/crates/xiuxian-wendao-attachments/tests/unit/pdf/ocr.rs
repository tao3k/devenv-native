use std::path::{Path, PathBuf};

use arrow::array::{Array, BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;

use super::super::render::{
    PdfPageBox, PdfPageRenderProfile, PdfPageShardManifest, PdfPageShardManifestInput,
    RenderedRasterIdentity, build_shard_manifest,
};
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

fn bool_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a BooleanArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| format!("`{name}` column is not Boolean"))
}

fn sample_manifest() -> PdfPageShardManifest {
    let profile = PdfPageRenderProfile::ocr_default();
    build_shard_manifest(PdfPageShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 3,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        rotation_degrees: 90,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00003.png"),
            sha256: "rasterhash".to_string(),
            width_px: 3100,
            height_px: 2400,
        },
    })
}

#[test]
fn document_extract_pdf_ocr_builds_worker_input_batch() -> Result<(), String> {
    let profile = PdfOcrWorkerProfile {
        profile_id: "ocr-profile".to_string(),
        engine: "fixture-engine".to_string(),
        preferred_languages: vec!["en".to_string(), "zh".to_string()],
        min_confidence: 0.65,
        preserve_layout: true,
    };
    let inputs = build_ocr_shard_inputs(&[sample_manifest()], &profile);
    let batch = build_ocr_shard_input_batch(&inputs)?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().field(0).name(), "contractVersion");
    assert_eq!(
        string_column(&batch, "contractVersion")?.value(0),
        PDF_OCR_SHARD_INPUT_SCHEMA_VERSION
    );
    assert_eq!(int32_column(&batch, "pageIndex")?.value(0), 3);
    assert_eq!(
        string_column(&batch, "preferredLanguages")?.value(0),
        "en,zh"
    );
    assert_eq!(
        string_column(&batch, "ocrEngine")?.value(0),
        "fixture-engine"
    );
    assert!(bool_column(&batch, "preserveLayout")?.value(0));
    assert_eq!(int32_column(&batch, "rotationDegrees")?.value(0), 90);
    assert_eq!(int32_column(&batch, "rasterWidthPx")?.value(0), 3100);
    assert_close(float64_column(&batch, "cropLeft")?.value(0), 18.0);
    Ok(())
}

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
