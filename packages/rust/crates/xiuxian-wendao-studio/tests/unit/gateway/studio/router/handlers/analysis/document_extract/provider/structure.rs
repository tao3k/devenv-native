#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    HybridDocumentResourceBatch, assert_close, build_document_structure_batch, fs,
    hybrid_document_structure_blocks, read_arrow_file, sample_ocr_input, sample_ocr_result,
    structure_float64_column, structure_string_column, test_resource_batch,
    validate_hybrid_precision_gate, write_hybrid_document_resource_artifacts,
};

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
    let metrics_path =
        output.join(xiuxian_wendao_attachments::pdf::metrics::DOCUMENT_METRICS_ARROW_CACHE_NAME);
    let metrics_batches = read_arrow_file(metrics_path.as_path())?;
    assert_eq!(metrics_batches.len(), 1);
    assert_eq!(metrics_batches[0].num_rows(), 0);
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
    let resource_batch = HybridDocumentResourceBatch::with_ocr(
        test_resource_batch(&[
            ("text_page", 0, "text-0"),
            ("ocr_text", 0, result.element_id.as_str()),
        ])?,
        vec![input],
        vec![result],
    );

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
    assert!(
        structure_string_column(&structure_batch, "provenance")?
            .value(1)
            .contains(r#""patchProtocol":"sentinel-sidecar-v1""#)
    );
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_precision_gate_rejects_error_resource_rows() -> Result<(), String> {
    let batch = test_resource_batch(&[("ocr_error", 0, "err-0")])?;
    let resource_batch = HybridDocumentResourceBatch::native(batch.clone());
    let blocks = hybrid_document_structure_blocks(&resource_batch, "sourcehash", "wendao-hybrid")?;

    let Err(error) = validate_hybrid_precision_gate(1, &[0], &batch, blocks.as_slice(), &[], &[])
    else {
        panic!("expected precision gate to reject error resource row");
    };

    assert!(error.contains("rejected resource row"));
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_precision_gate_requires_ocr_bbox_provenance() -> Result<(), String> {
    let input = sample_ocr_input(0, "page");
    let result = sample_ocr_result(0, true);
    let batch = test_resource_batch(&[("ocr_text", 0, result.element_id.as_str())])?;
    let resource_batch = HybridDocumentResourceBatch::with_ocr(
        batch.clone(),
        vec![input.clone()],
        vec![result.clone()],
    );
    let mut blocks =
        hybrid_document_structure_blocks(&resource_batch, "sourcehash", "wendao-hybrid")?;
    blocks[0].bbox_left = None;

    let Err(error) =
        validate_hybrid_precision_gate(1, &[], &batch, blocks.as_slice(), &[input], &[result])
    else {
        panic!("expected precision gate to reject OCR block without bbox");
    };

    assert!(error.contains("missing bbox provenance"));
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_precision_gate_requires_region_sentinel_patch_protocol() -> Result<(), String> {
    let parent = sample_ocr_input(0, "page");
    let mut region = sample_ocr_input(0, "region");
    region.shard_element_id = "region-shard-0".to_string();
    region.parent_shard_element_id = parent.shard_element_id.clone();
    let parent_result = sample_ocr_result(0, true);
    let mut region_result = sample_ocr_result(0, true);
    region_result.shard_element_id = region.shard_element_id.clone();
    region_result.element_id = "region-result-0".to_string();
    let batch = test_resource_batch(&[
        ("ocr_text", 0, parent_result.element_id.as_str()),
        ("ocr_text", 0, region_result.element_id.as_str()),
    ])?;
    let resource_batch = HybridDocumentResourceBatch::with_ocr(
        batch.clone(),
        vec![parent.clone(), region.clone()],
        vec![parent_result.clone(), region_result.clone()],
    );
    let mut blocks =
        hybrid_document_structure_blocks(&resource_batch, "sourcehash", "wendao-hybrid")?;
    let region_block = blocks
        .iter_mut()
        .find(|block| block.block_type == "ocr_region")
        .ok_or_else(|| "missing region block".to_string())?;
    region_block.provenance = serde_json::json!({
        "source": "pdf_ocr_shard",
        "shardType": "region",
        "shardElementId": region.shard_element_id.clone(),
        "parentShardElementId": region.parent_shard_element_id.clone(),
    })
    .to_string();

    let Err(error) = validate_hybrid_precision_gate(
        1,
        &[],
        &batch,
        blocks.as_slice(),
        &[parent, region],
        &[parent_result, region_result],
    ) else {
        panic!("expected precision gate to reject missing region patch protocol");
    };

    assert!(error.contains("missing sentinel patch protocol"));
    Ok(())
}
