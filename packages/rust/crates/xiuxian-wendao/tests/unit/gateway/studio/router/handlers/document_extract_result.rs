use std::sync::Arc as StdArc;

use arrow::array::{ArrayRef, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::FileWriter;

use super::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, File, RecordBatch, document_extract_resources_from_batch,
    document_source_format, read_document_extract_resources, total_pages_from_resources,
};

#[test]
fn document_source_format_normalizes_common_docling_suffixes() {
    assert_eq!(document_source_format("docs/manual.DOCX"), "docx");
    assert_eq!(document_source_format("slides/report.pptx"), "pptx");
    assert_eq!(document_source_format("no-extension"), "unknown");
}

#[test]
fn document_extract_resources_preserve_resource_count_and_page_span() {
    let batch = document_resource_batch();

    let resources = document_extract_resources_from_batch(&batch)
        .unwrap_or_else(|error| panic!("decode resources: {error:?}"));

    assert_eq!(resources.len(), 2);
    assert_eq!(total_pages_from_resources(resources.as_slice()), 3);
    assert_eq!(resources[1].resource_type, "table");
}

#[test]
fn read_document_extract_resources_decodes_arrow_ipc_file() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let resources_path = temp_dir.path().join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
    let batch = document_resource_batch();
    let file = File::create(&resources_path).unwrap_or_else(|error| panic!("file: {error}"));
    let mut writer = FileWriter::try_new(file, &batch.schema())
        .unwrap_or_else(|error| panic!("writer: {error}"));
    writer
        .write(&batch)
        .unwrap_or_else(|error| panic!("write batch: {error}"));
    writer
        .finish()
        .unwrap_or_else(|error| panic!("finish: {error}"));

    let resources = read_document_extract_resources(&resources_path)
        .unwrap_or_else(|error| panic!("read resources: {error:?}"));

    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0].element_id, "_main");
}

fn document_resource_batch() -> RecordBatch {
    let schema = StdArc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, false),
        Field::new("resourceType", DataType::Utf8, false),
        Field::new("resourcePath", DataType::Utf8, false),
        Field::new("pageIndex", DataType::Int32, false),
        Field::new("caption", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("mimeType", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("elementId", DataType::Utf8, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            StdArc::new(StringArray::from(vec!["manual.docx", "manual.docx"])) as ArrayRef,
            StdArc::new(StringArray::from(vec!["document", "table"])) as ArrayRef,
            StdArc::new(StringArray::from(vec![
                "manual.docx.extracted/manual.md",
                "manual.docx.extracted/table-2.csv",
            ])) as ArrayRef,
            StdArc::new(Int32Array::from(vec![0, 2])) as ArrayRef,
            StdArc::new(StringArray::from(vec!["", ""])) as ArrayRef,
            StdArc::new(StringArray::from(vec!["# Manual", ""])) as ArrayRef,
            StdArc::new(StringArray::from(vec!["text/markdown", "text/csv"])) as ArrayRef,
            StdArc::new(StringArray::from(vec!["ok", "ok"])) as ArrayRef,
            StdArc::new(StringArray::from(vec!["_main", "table-2"])) as ArrayRef,
        ],
    )
    .unwrap_or_else(|error| panic!("batch: {error}"))
}
