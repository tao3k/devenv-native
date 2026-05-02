use super::{
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrWorkerProfile, assert_close, bool_column,
    build_ocr_shard_input_batch, build_ocr_shard_inputs, decode_ocr_shard_input_batch,
    float64_column, int32_column, sample_manifest, sample_region_manifest, string_column,
};

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
    assert_eq!(string_column(&batch, "shardType")?.value(0), "page");
    assert_eq!(int32_column(&batch, "regionIndex")?.value(0), 0);
    assert_eq!(
        string_column(&batch, "readingOrderKey")?.value(0),
        "000003.000000"
    );
    assert_eq!(int32_column(&batch, "sourcePagePixelRight")?.value(0), 3100);
    Ok(())
}

#[test]
fn document_extract_pdf_ocr_builds_region_worker_input_batch() -> Result<(), String> {
    let inputs = build_ocr_shard_inputs(
        &[sample_region_manifest()?],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    let batch = build_ocr_shard_input_batch(&inputs)?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(string_column(&batch, "shardType")?.value(0), "region");
    assert_eq!(int32_column(&batch, "regionIndex")?.value(0), 4);
    assert_eq!(
        string_column(&batch, "readingOrderKey")?.value(0),
        "000003.000004"
    );
    assert_eq!(int32_column(&batch, "sourcePagePixelLeft")?.value(0), 775);
    assert_eq!(int32_column(&batch, "sourcePagePixelTop")?.value(0), 1200);
    Ok(())
}

#[test]
fn document_extract_pdf_ocr_decodes_worker_input_batch() -> Result<(), String> {
    let profile = PdfOcrWorkerProfile {
        profile_id: "ocr-profile".to_string(),
        engine: "fixture-engine".to_string(),
        preferred_languages: vec!["en".to_string(), "zh".to_string()],
        min_confidence: 0.65,
        preserve_layout: true,
    };
    let inputs = build_ocr_shard_inputs(&[sample_manifest()], &profile);
    let batch = build_ocr_shard_input_batch(&inputs)?;

    let decoded = decode_ocr_shard_input_batch(&batch)?;

    assert_eq!(decoded, inputs);
    Ok(())
}
