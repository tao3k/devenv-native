use crate::link_graph::LinkGraphAttachmentKind;
use crate::link_graph::models::{LinkGraphAttachment, VisionAnnotation};
use crate::parsers::markdown::ParsedNote;
use arrow::array::{Array, Int32Array, StringArray};
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn attachment_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map_or_else(|| path.to_string(), ToString::to_string)
}

fn attachment_ext(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().trim_start_matches('.').to_lowercase())
        .unwrap_or_default()
}

pub(super) fn attachments_for_parsed_note(parsed: &ParsedNote) -> Vec<LinkGraphAttachment> {
    let mut rows: Vec<LinkGraphAttachment> = parsed
        .attachment_targets
        .iter()
        .map(|attachment_path| {
            let ext = attachment_ext(attachment_path);
            LinkGraphAttachment {
                source_id: parsed.doc.id.clone(),
                source_stem: parsed.doc.stem.clone(),
                source_path: parsed.doc.path.clone(),
                source_title: parsed.doc.title.clone(),
                attachment_path: attachment_path.clone(),
                attachment_name: attachment_name(attachment_path),
                attachment_ext: ext.clone(),
                kind: LinkGraphAttachmentKind::from_extension(&ext),
                vision_annotation: None,
            }
        })
        .collect();
    rows.sort_by(|left, right| {
        left.attachment_path
            .cmp(&right.attachment_path)
            .then(left.source_path.cmp(&right.source_path))
    });
    rows.dedup_by(|left, right| {
        left.source_id == right.source_id && left.attachment_path == right.attachment_path
    });
    rows
}

const DOCLING_DOCUMENT_SOURCE_EXTENSIONS: &[&str] = &[
    "pdf", "docx", "xlsx", "pptx", "md", "markdown", "adoc", "asciidoc", "html", "htm", "xhtml",
    "csv", "png", "jpg", "jpeg", "tif", "tiff", "bmp", "webp", "xml", "json",
];
const DOCUMENT_RESOURCE_ARROW_CACHE_NAME: &str = "_resources.arrow";

/// Expand document attachments into derived resources from cached extraction rows.
///
/// Reads `{document_path}.extracted/_resources.arrow` for Docling-supported
/// source attachments. Derived resources are added as additional
/// `LinkGraphAttachment` rows so they appear in VFS scans and attachment
/// search.
pub(super) fn expand_document_attachments(rows: &mut Vec<LinkGraphAttachment>) {
    let document_rows: Vec<LinkGraphAttachment> = rows
        .iter()
        .filter(|row| is_docling_document_source(row))
        .cloned()
        .collect();

    for document in &document_rows {
        let extracted_dir = format!("{}.extracted", document.attachment_path);
        let resources_path = Path::new(&extracted_dir).join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
        let source_path = Path::new(&document.attachment_path);

        if !resources_path.exists() {
            continue;
        }

        if cache_is_stale(source_path, &resources_path) {
            continue;
        }

        let Some(resources) = read_cached_resources(&resources_path) else {
            continue;
        };

        let mut derived: Vec<LinkGraphAttachment> = resources
            .iter()
            .filter_map(|resource| derived_attachment_from_resource(document, resource))
            .collect();

        for (index, derived_attachment) in derived.iter_mut().enumerate() {
            if index == 0 && path_has_extension(&derived_attachment.attachment_path, "md") {
                continue;
            }
            if derived_attachment.kind == LinkGraphAttachmentKind::Image {
                derived_attachment.vision_annotation = Some(VisionAnnotation {
                    description: format!("Extracted document image {}", index + 1),
                    confidence: 0.95,
                    entities: Vec::new(),
                    annotated_at: unix_now_i64(),
                });
            }
        }

        rows.extend(derived);
    }

    rows.sort_by(|left, right| {
        left.attachment_path
            .cmp(&right.attachment_path)
            .then(left.source_path.cmp(&right.source_path))
    });
    rows.dedup_by(|left, right| {
        left.source_id == right.source_id && left.attachment_path == right.attachment_path
    });
}

fn is_docling_document_source(row: &LinkGraphAttachment) -> bool {
    DOCLING_DOCUMENT_SOURCE_EXTENSIONS
        .iter()
        .any(|extension| row.attachment_ext.eq_ignore_ascii_case(extension))
}

fn cache_is_stale(source_path: &Path, resources_path: &Path) -> bool {
    let source_mtime = source_path.metadata().and_then(|m| m.modified()).ok();
    let resources_mtime = resources_path.metadata().and_then(|m| m.modified()).ok();
    matches!((source_mtime, resources_mtime), (Some(source), Some(resources)) if source > resources)
}

fn read_cached_resources(resources_path: &Path) -> Option<Vec<DocumentResourceCacheRow>> {
    let file = File::open(resources_path).ok()?;
    let reader = FileReader::try_new(file, None).ok()?;
    let mut resources = Vec::new();
    for batch in reader {
        resources.extend(resources_from_batch(&batch.ok()?)?);
    }
    Some(resources)
}

#[derive(Debug, Clone)]
struct DocumentResourceCacheRow {
    resource_type: String,
    resource_path: String,
    content: String,
    status: String,
}

fn resources_from_batch(batch: &RecordBatch) -> Option<Vec<DocumentResourceCacheRow>> {
    let resource_type = string_column(batch, "resourceType")?;
    let resource_path = string_column(batch, "resourcePath")?;
    let content = string_column(batch, "content")?;
    let status = string_column(batch, "status")?;
    let _page_index = i32_column(batch, "pageIndex")?;

    let mut resources = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        resources.push(DocumentResourceCacheRow {
            resource_type: string_value(resource_type, row, "document").to_string(),
            resource_path: string_value(resource_path, row, "").to_string(),
            content: string_value(content, row, "").to_string(),
            status: string_value(status, row, "ok").to_string(),
        });
    }
    Some(resources)
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Option<&'a StringArray> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
}

fn i32_column<'a>(batch: &'a RecordBatch, name: &str) -> Option<&'a Int32Array> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
}

fn string_value<'a>(array: &'a StringArray, row: usize, default: &'a str) -> &'a str {
    if array.is_null(row) {
        default
    } else {
        array.value(row)
    }
}

fn derived_attachment_from_resource(
    source: &LinkGraphAttachment,
    resource: &DocumentResourceCacheRow,
) -> Option<LinkGraphAttachment> {
    let resource_path = resource.resource_path.as_str();
    if resource_path.is_empty() || resource.status == "error" {
        return None;
    }
    let resource_type = resource.resource_type.as_str();
    let content = resource.content.as_str();
    let ext = attachment_ext(resource_path);

    Some(LinkGraphAttachment {
        source_id: source.source_id.clone(),
        source_stem: source.source_stem.clone(),
        source_path: source.source_path.clone(),
        source_title: source.source_title.clone(),
        attachment_path: resource_path.to_string(),
        attachment_name: attachment_name(resource_path),
        attachment_ext: ext,
        kind: attachment_kind_for_resource_type(resource_type),
        vision_annotation: vision_annotation_for_content(content),
    })
}

fn attachment_kind_for_resource_type(resource_type: &str) -> LinkGraphAttachmentKind {
    match resource_type {
        "image" => LinkGraphAttachmentKind::Image,
        "table" | "formula" | "document" => LinkGraphAttachmentKind::Document,
        _ => LinkGraphAttachmentKind::Other,
    }
}

fn vision_annotation_for_content(content: &str) -> Option<VisionAnnotation> {
    if content.is_empty() {
        return None;
    }
    Some(VisionAnnotation {
        description: content.to_string(),
        confidence: 0.95,
        entities: Vec::new(),
        annotated_at: unix_now_i64(),
    })
}

fn unix_now_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
        })
}

fn path_has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/build/attachments.rs"]
mod tests;
