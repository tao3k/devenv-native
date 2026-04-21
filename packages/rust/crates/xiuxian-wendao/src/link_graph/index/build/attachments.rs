use crate::link_graph::LinkGraphAttachmentKind;
use crate::link_graph::models::{LinkGraphAttachment, VisionAnnotation};
use crate::parsers::markdown::ParsedNote;
use std::collections::HashMap;
use std::path::Path;

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

/// Expand PDF attachments into derived resources from cached extraction metadata.
///
/// Reads `{pdf_path}.extracted/_metadata.json` if it exists and is newer than the
/// source PDF.  Derived resources (markdown, images, tables, formulas) are added
/// as additional `LinkGraphAttachment` rows so they appear in VFS scans and
/// attachment search.
pub(super) fn expand_pdf_attachments(rows: &mut Vec<LinkGraphAttachment>) {
    let pdf_rows: Vec<LinkGraphAttachment> = rows
        .iter()
        .filter(|row| row.kind == LinkGraphAttachmentKind::Pdf)
        .cloned()
        .collect();

    for pdf in &pdf_rows {
        let extracted_dir = format!("{}.extracted", pdf.attachment_path);
        let metadata_path = Path::new(&extracted_dir).join("_metadata.json");
        let source_path = Path::new(&pdf.attachment_path);

        if !metadata_path.exists() {
            continue;
        }

        let source_mtime = source_path.metadata().and_then(|m| m.modified()).ok();
        let meta_mtime = metadata_path.metadata().and_then(|m| m.modified()).ok();

        if let (Some(s), Some(m)) = (source_mtime, meta_mtime) {
            if s > m {
                // Source is newer than cache — skip stale cache
                continue;
            }
        }

        let metadata_content = match std::fs::read_to_string(&metadata_path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let resources: Vec<HashMap<String, serde_json::Value>> =
            match serde_json::from_str(&metadata_content) {
                Ok(r) => r,
                Err(_) => continue,
            };

        let mut derived: Vec<LinkGraphAttachment> = Vec::new();
        for res in &resources {
            let resource_type = res
                .get("resourceType")
                .and_then(|v| v.as_str())
                .unwrap_or("document");
            let resource_path = res
                .get("resourcePath")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let _page_index = res.get("pageIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
            let _caption = res
                .get("caption")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let content = res
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let _mime_type = res
                .get("mimeType")
                .and_then(|v| v.as_str())
                .unwrap_or("text/plain");
            let status = res
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("ok");

            if status == "error" || resource_path.is_empty() {
                continue;
            }

            let kind = match resource_type {
                "image" => LinkGraphAttachmentKind::Image,
                "table" => LinkGraphAttachmentKind::Document,
                "formula" => LinkGraphAttachmentKind::Document,
                "document" => LinkGraphAttachmentKind::Document,
                _ => LinkGraphAttachmentKind::Other,
            };

            let ext = attachment_ext(resource_path);
            let name = attachment_name(resource_path);

            let vision_annotation = if !content.is_empty() {
                Some(VisionAnnotation {
                    description: content.to_string(),
                    confidence: 0.95,
                    entities: Vec::new(),
                    annotated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs() as i64),
                })
            } else {
                None
            };

            derived.push(LinkGraphAttachment {
                source_id: pdf.source_id.clone(),
                source_stem: pdf.source_stem.clone(),
                source_path: pdf.source_path.clone(),
                source_title: pdf.source_title.clone(),
                attachment_path: resource_path.to_string(),
                attachment_name: name,
                attachment_ext: ext,
                kind,
                vision_annotation,
            });
        }

        // Inject page index into caption for derived resources
        for (idx, d) in derived.iter_mut().enumerate() {
            if idx == 0 && d.attachment_path.ends_with(".md") {
                // Main markdown document — keep caption empty
                continue;
            }
            if d.kind == LinkGraphAttachmentKind::Image {
                d.vision_annotation = Some(VisionAnnotation {
                    description: format!("PDF page {} image", d.vision_annotation.as_ref().map_or(0, |v| v.annotated_at)),
                    confidence: 0.95,
                    entities: Vec::new(),
                    annotated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs() as i64),
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
