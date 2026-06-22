use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use crate::pdf::structure::{
    DOCUMENT_STRUCTURE_SCHEMA_VERSION, DocumentStructureBlock, build_document_structure_batch,
    document_resource_batch_to_structure_blocks, document_structure_schema,
    validate_document_structure_parity,
};
use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

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

fn typed_block(
    block_id: &str,
    page_index: i32,
    block_index: i32,
    block_type: &str,
    content: &str,
) -> DocumentStructureBlock {
    let mut block = block(block_id, page_index, block_index);
    block.block_type = block_type.to_string();
    block.content = content.to_string();
    block
}

#[test]
fn document_extract_structure_batch_uses_stable_schema_and_order() -> Result<(), String> {
    let mut second = block("second", 1, 1);
    second.confidence = Some(0.92);
    second.bbox_left = Some(1.0);
    let batch = build_document_structure_batch(&[second, block("first", 0, 0)])?;

    assert_eq!(batch.schema(), document_structure_schema());
    assert_eq!(
        batch
            .schema()
            .metadata()
            .get(WENDAO_TABLE_METADATA_KEY)
            .map(String::as_str),
        Some("pdf_document_structure")
    );
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

#[test]
fn document_structure_parity_accepts_candidate_with_full_baseline_coverage() -> Result<(), String> {
    let baseline = vec![
        typed_block("text-0", 0, 0, "text_page", "alpha beta"),
        typed_block("table-0", 0, 1, "table", "| a | b |"),
        typed_block("formula-1", 1, 0, "formula", "x = y"),
    ];
    let candidate = vec![
        typed_block("text-0", 0, 0, "text_page", "alpha beta plus"),
        typed_block("table-0", 0, 1, "table", "| a | b |"),
        typed_block("formula-1", 1, 0, "formula", "x = y"),
        typed_block("ocr-1", 1, 1, "ocr_page", "extra text"),
    ];

    let summary = validate_document_structure_parity(baseline.as_slice(), candidate.as_slice())?;

    assert_eq!(summary.baseline_block_count, 3);
    assert_eq!(summary.candidate_block_count, 4);
    assert_eq!(summary.baseline_page_count, 2);
    assert_eq!(summary.candidate_page_count, 2);
    assert_eq!(
        summary
            .protected_block_counts
            .get("table")
            .ok_or_else(|| "missing table parity count".to_string())?
            .candidate,
        1
    );
    Ok(())
}

#[test]
fn document_structure_parity_rejects_missing_page() {
    let baseline = vec![
        typed_block("text-0", 0, 0, "text_page", "alpha"),
        typed_block("text-1", 1, 0, "text_page", "beta"),
    ];
    let candidate = vec![typed_block("text-0", 0, 0, "text_page", "alpha beta")];

    let Err(error) = validate_document_structure_parity(baseline.as_slice(), candidate.as_slice())
    else {
        panic!("missing page should fail parity");
    };

    assert!(error.contains("missing baseline pages: 1"));
}

#[test]
fn document_structure_parity_rejects_lower_page_text_coverage() {
    let baseline = vec![typed_block("text-0", 0, 0, "text_page", "abcdef")];
    let candidate = vec![typed_block("text-0", 0, 0, "text_page", "abc")];

    let Err(error) = validate_document_structure_parity(baseline.as_slice(), candidate.as_slice())
    else {
        panic!("lower text coverage should fail parity");
    };

    assert!(error.contains("text chars, below baseline"));
}

#[test]
fn document_structure_parity_ignores_document_wrapper_text() -> Result<(), String> {
    let baseline = vec![
        typed_block("docling-json", 0, 0, "docling_json", "full document json"),
        typed_block("document", 0, 1, "document", "full document markdown"),
        typed_block("table-0", 0, 2, "table", "| a | b |"),
    ];
    let candidate = vec![
        typed_block("page-json", 0, 0, "docling_json", "page json"),
        typed_block("page-document", 0, 1, "document", "page markdown"),
        typed_block("table-0", 0, 2, "table", "| a | b |"),
    ];

    let summary = validate_document_structure_parity(baseline.as_slice(), candidate.as_slice())?;

    assert_eq!(summary.baseline_text_chars, 5);
    assert_eq!(summary.candidate_text_chars, 5);
    Ok(())
}

#[test]
fn document_structure_parity_rejects_protected_block_loss() {
    let baseline = vec![
        typed_block("text-0", 0, 0, "text_page", "alpha"),
        typed_block("table-0", 0, 1, "table", "| a | b |"),
    ];
    let candidate = vec![
        typed_block("text-0", 0, 0, "text_page", "alpha"),
        typed_block("ocr-0", 0, 1, "ocr_page", "| a | b |"),
    ];

    let Err(error) = validate_document_structure_parity(baseline.as_slice(), candidate.as_slice())
    else {
        panic!("protected block loss should fail parity");
    };

    assert!(error.contains("`table` blocks"));
}

#[test]
fn document_structure_parity_rejects_unsorted_candidate_order() {
    let baseline = vec![
        typed_block("text-0", 0, 0, "text_page", "alpha"),
        typed_block("text-1", 1, 0, "text_page", "beta"),
    ];
    let candidate = vec![
        typed_block("text-1", 1, 0, "text_page", "alpha beta"),
        typed_block("text-0", 0, 0, "text_page", "alpha beta"),
    ];

    let Err(error) = validate_document_structure_parity(baseline.as_slice(), candidate.as_slice())
    else {
        panic!("unsorted candidate should fail parity");
    };

    assert!(error.contains("candidate is not sorted"));
}
