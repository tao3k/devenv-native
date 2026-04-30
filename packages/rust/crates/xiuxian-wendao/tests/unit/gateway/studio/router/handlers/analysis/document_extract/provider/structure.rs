use super::*;

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_writes_structure_sidecar() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    let output = temp.path().join("out");
    fs::write(source.as_path(), b"%PDF-1.4\n").map_err(|error| error.to_string())?;
    fs::create_dir_all(output.as_path()).map_err(|error| error.to_string())?;
    let resource_batch =
        HybridDocumentResourceBatch::native(test_resource_batch(&[("text_page", 0, "text-0")])?);

    write_hybrid_document_resource_artifacts(output.as_path(), source.as_path(), &resource_batch)?;

    let structure_path = output
        .join(xiuxian_wendao_attachments::pdf::structure::DOCUMENT_STRUCTURE_ARROW_CACHE_NAME);
    let structure_batches = read_arrow_file(structure_path.as_path())?;
    assert_eq!(structure_batches.len(), 1);
    assert_eq!(structure_batches[0].num_rows(), 1);
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_structure_sidecar_preserves_region_provenance() -> Result<(), String> {
    let mut input = sample_ocr_input(0, "region");
    input.crop_left = 72.0;
    input.crop_bottom = 80.0;
    input.crop_right = 240.0;
    input.crop_top = 320.0;
    input.reading_order_key = "000000.000004".to_string();
    let mut result = sample_ocr_result(0, true);
    result.confidence = Some(0.87);
    let resource_batch = HybridDocumentResourceBatch {
        batch: test_resource_batch(&[
            ("text_page", 0, "text-0"),
            ("ocr_text", 0, result.element_id.as_str()),
        ])?,
        ocr_inputs: vec![input],
        ocr_results: vec![result],
    };

    let blocks = hybrid_document_structure_blocks(&resource_batch, "sourcehash", "wendao-hybrid")?;
    let structure_batch = build_document_structure_batch(blocks.as_slice())?;

    assert_eq!(
        structure_string_column(&structure_batch, "blockId")?.value(1),
        "ocr-0"
    );
    assert_eq!(
        structure_string_column(&structure_batch, "readingOrderKey")?.value(1),
        "000000.000004"
    );
    assert_eq!(
        structure_string_column(&structure_batch, "blockType")?.value(1),
        "ocr_region"
    );
    assert_eq!(
        structure_string_column(&structure_batch, "parentBlockId")?.value(1),
        "page-shard-0"
    );
    assert_close(
        structure_float64_column(&structure_batch, "bboxLeft")?.value(1),
        72.0,
    );
    assert_close(
        structure_float64_column(&structure_batch, "bboxBottom")?.value(1),
        80.0,
    );
    assert_close(
        structure_float64_column(&structure_batch, "bboxRight")?.value(1),
        240.0,
    );
    assert_close(
        structure_float64_column(&structure_batch, "bboxTop")?.value(1),
        320.0,
    );
    assert_close(
        structure_float64_column(&structure_batch, "confidence")?.value(1),
        0.87,
    );
    assert!(
        structure_string_column(&structure_batch, "provenance")?
            .value(1)
            .contains(r#""shardType":"region""#)
    );
    Ok(())
}
