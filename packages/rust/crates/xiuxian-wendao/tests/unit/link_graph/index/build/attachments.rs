use std::sync::Arc;

use arrow::array::{ArrayRef, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use std::fs::File;
use std::path::Path;

use super::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, DocumentResourceCacheRow, attachment_name,
    expand_document_attachments,
};
use crate::link_graph::LinkGraphAttachmentKind;
use crate::link_graph::models::LinkGraphAttachment;

#[test]
fn expand_document_attachments_uses_docling_supported_source_suffixes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let source_path = temp_dir.path().join("manual.docx");
    std::fs::write(&source_path, b"docx fixture")
        .unwrap_or_else(|error| panic!("write source: {error}"));
    write_resource_cache(
        &source_path,
        &[DocumentResourceCacheRow {
            resource_type: "document".to_string(),
            resource_path: "manual.docx.extracted/manual.md".to_string(),
            content: "# Manual".to_string(),
            status: "ok".to_string(),
        }],
    );
    let mut rows = vec![attachment_row(
        &source_path,
        "docx",
        LinkGraphAttachmentKind::Document,
    )];

    expand_document_attachments(&mut rows);

    assert!(
        rows.iter()
            .any(|row| row.attachment_path == "manual.docx.extracted/manual.md")
    );
    assert_eq!(
        rows.iter()
            .find(|row| row.attachment_path == "manual.docx.extracted/manual.md")
            .map(|row| row.kind),
        Some(LinkGraphAttachmentKind::Document)
    );
}

#[test]
fn expand_document_attachments_ignores_non_docling_source_suffixes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let source_path = temp_dir.path().join("bundle.zip");
    std::fs::write(&source_path, b"zip fixture")
        .unwrap_or_else(|error| panic!("write source: {error}"));
    write_resource_cache(
        &source_path,
        &[DocumentResourceCacheRow {
            resource_type: "document".to_string(),
            resource_path: "bundle.zip.extracted/bundle.md".to_string(),
            content: String::new(),
            status: "ok".to_string(),
        }],
    );
    let mut rows = vec![attachment_row(
        &source_path,
        "zip",
        LinkGraphAttachmentKind::Archive,
    )];

    expand_document_attachments(&mut rows);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].attachment_path, source_path.to_string_lossy());
}

fn write_resource_cache(source_path: &Path, resources: &[DocumentResourceCacheRow]) {
    let extracted_dir = Path::new(&format!("{}.extracted", source_path.display())).to_path_buf();
    std::fs::create_dir_all(&extracted_dir)
        .unwrap_or_else(|error| panic!("create extracted dir: {error}"));
    let batch = resource_batch(source_path, resources);
    let file = File::create(extracted_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME))
        .unwrap_or_else(|error| panic!("create resources cache: {error}"));
    let mut writer = FileWriter::try_new(file, &batch.schema())
        .unwrap_or_else(|error| panic!("create writer: {error}"));
    writer
        .write(&batch)
        .unwrap_or_else(|error| panic!("write batch: {error}"));
    writer
        .finish()
        .unwrap_or_else(|error| panic!("finish: {error}"));
}

fn resource_batch(source_path: &Path, resources: &[DocumentResourceCacheRow]) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
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
    let source_paths = vec![source_path.to_string_lossy().into_owned(); resources.len()];
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(source_paths)) as ArrayRef,
            Arc::new(StringArray::from(
                resources
                    .iter()
                    .map(|resource| resource.resource_type.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(
                resources
                    .iter()
                    .map(|resource| resource.resource_path.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(Int32Array::from(vec![0; resources.len()])) as ArrayRef,
            Arc::new(StringArray::from(vec![""; resources.len()])) as ArrayRef,
            Arc::new(StringArray::from(
                resources
                    .iter()
                    .map(|resource| resource.content.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(vec!["text/markdown"; resources.len()])) as ArrayRef,
            Arc::new(StringArray::from(
                resources
                    .iter()
                    .map(|resource| resource.status.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(vec!["_main"; resources.len()])) as ArrayRef,
        ],
    )
    .unwrap_or_else(|error| panic!("resource batch: {error}"))
}

fn attachment_row(
    source_path: &Path,
    extension: &str,
    kind: LinkGraphAttachmentKind,
) -> LinkGraphAttachment {
    LinkGraphAttachment {
        source_id: "docs/alpha".to_string(),
        source_stem: "alpha".to_string(),
        source_path: "docs/alpha.md".to_string(),
        source_title: "Alpha".to_string(),
        attachment_path: source_path.to_string_lossy().into_owned(),
        attachment_name: attachment_name(&source_path.to_string_lossy()),
        attachment_ext: extension.to_string(),
        kind,
        vision_annotation: None,
    }
}
