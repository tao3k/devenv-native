use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "expected {actual} to be close to {expected}"
    );
}

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

fn block(block_id: &str, page_index: i32, block_index: i32) -> DocumentStructureBlock {
    DocumentStructureBlock {
        contract_version: DOCUMENT_STRUCTURE_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "hash".to_string(),
        block_id: block_id.to_string(),
        parent_block_id: String::new(),
        page_index,
        block_index,
        reading_order_key: format!("{page_index:06}.{block_index:06}"),
        block_type: "text_page".to_string(),
        resource_element_id: block_id.to_string(),
        content: block_id.to_string(),
        mime_type: "text/markdown".to_string(),
        status: "ok".to_string(),
        engine: "wendao-hybrid".to_string(),
        confidence: None,
        bbox_left: None,
        bbox_top: None,
        bbox_right: None,
        bbox_bottom: None,
        provenance: "{}".to_string(),
    }
}

#[test]
fn document_extract_structure_batch_uses_stable_schema_and_order() -> Result<(), String> {
    let mut second = block("second", 1, 1);
    second.confidence = Some(0.92);
    second.bbox_left = Some(1.0);
    let batch = build_document_structure_batch(&[second, block("first", 0, 0)])?;

    assert_eq!(batch.schema(), document_structure_schema());
    assert_eq!(batch.num_rows(), 2);
    assert_eq!(string_column(&batch, "blockId")?.value(0), "first");
    assert_eq!(string_column(&batch, "blockId")?.value(1), "second");
    assert_eq!(
        string_column(&batch, "contractVersion")?.value(0),
        DOCUMENT_STRUCTURE_SCHEMA_VERSION
    );
    assert!(float64_column(&batch, "confidence")?.is_null(0));
    assert_close(float64_column(&batch, "confidence")?.value(1), 0.92);
    assert_close(float64_column(&batch, "bboxLeft")?.value(1), 1.0);
    Ok(())
}

#[test]
fn document_extract_structure_projects_resource_rows_without_json_contract() -> Result<(), String> {
    let resources = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("sourcePath", DataType::Utf8, true),
            Field::new("resourceType", DataType::Utf8, true),
            Field::new("resourcePath", DataType::Utf8, true),
            Field::new("pageIndex", DataType::Int32, true),
            Field::new("caption", DataType::Utf8, true),
            Field::new("content", DataType::Utf8, true),
            Field::new("mimeType", DataType::Utf8, true),
            Field::new("status", DataType::Utf8, true),
            Field::new("elementId", DataType::Utf8, true),
        ])),
        vec![
            Arc::new(StringArray::from(vec![
                "/tmp/source.pdf",
                "/tmp/source.pdf",
            ])),
            Arc::new(StringArray::from(vec!["ocr_text", "text_page"])),
            Arc::new(StringArray::from(vec!["/tmp/page.png", "/tmp/source.pdf"])),
            Arc::new(Int32Array::from(vec![2, 1])),
            Arc::new(StringArray::from(vec!["OCR", "Text"])),
            Arc::new(StringArray::from(vec!["recognized", "native"])),
            Arc::new(StringArray::from(vec!["text/markdown", "text/markdown"])),
            Arc::new(StringArray::from(vec!["succeeded", "ok"])),
            Arc::new(StringArray::from(vec!["ocr-2", "text-1"])),
        ],
    )
    .map_err(|error| error.to_string())?;

    let blocks =
        document_resource_batch_to_structure_blocks(&resources, "sourcehash", "wendao-hybrid")?;
    let batch = build_document_structure_batch(blocks.as_slice())?;

    assert_eq!(string_column(&batch, "blockId")?.value(0), "text-1");
    assert_eq!(string_column(&batch, "blockId")?.value(1), "ocr-2");
    assert_eq!(
        string_column(&batch, "sourceContentHash")?.value(0),
        "sourcehash"
    );
    assert_eq!(string_column(&batch, "engine")?.value(1), "wendao-hybrid");
    assert_eq!(
        string_column(&batch, "provenance")?.value(1),
        r#"{"rowIndex":0,"source":"document_resource_batch"}"#
    );
    assert_eq!(int32_column(&batch, "blockIndex")?.value(1), 0);
    Ok(())
}
