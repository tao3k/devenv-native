use std::path::Path;

use crate::link_graph::LinkGraphAttachmentKind;
use crate::parsers::markdown::ParsedNote;
use crate::search::MarkdownSnapshotEntry;
use crate::search::contracts::{AttachmentSearchHit, StudioNavigationTarget};
use std::collections::HashSet;

pub(crate) fn build_attachment_hits_for_entry(
    entry: &MarkdownSnapshotEntry,
) -> Vec<AttachmentSearchHit> {
    let Some(parsed) = entry.parsed_note.as_ref() else {
        return Vec::new();
    };
    attachment_hits_for_parsed_note(
        parsed,
        entry.file.project_name.as_deref(),
        entry.file.root_label.as_deref(),
    )
}

fn attachment_hits_for_parsed_note(
    parsed: &ParsedNote,
    project_name: Option<&str>,
    root_label: Option<&str>,
) -> Vec<AttachmentSearchHit> {
    let mut seen = HashSet::<String>::new();
    let mut hits = parsed
        .attachment_targets
        .iter()
        .filter(|attachment_path| seen.insert((*attachment_path).clone()))
        .map(|attachment_path| {
            let attachment_name = attachment_name(attachment_path);
            let attachment_ext = attachment_ext(attachment_path);
            AttachmentSearchHit {
                name: attachment_name.clone(),
                path: parsed.doc.path.clone(),
                source_id: parsed.doc.id.clone(),
                source_stem: parsed.doc.stem.clone(),
                source_title: parsed.doc.title.clone(),
                source_path: parsed.doc.path.clone(),
                attachment_id: format!("att://{}/{}", parsed.doc.id, attachment_path),
                attachment_path: attachment_path.clone(),
                attachment_name,
                attachment_ext: attachment_ext.clone(),
                kind: attachment_kind_label(LinkGraphAttachmentKind::from_extension(
                    attachment_ext.as_str(),
                ))
                .to_string(),
                navigation_target: StudioNavigationTarget {
                    path: parsed.doc.path.clone(),
                    category: "doc".to_string(),
                    project_name: project_name.map(ToString::to_string),
                    root_label: root_label.map(ToString::to_string),
                    line: None,
                    line_end: None,
                    column: None,
                },
                score: 0.0,
                vision_snippet: None,
            }
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        left.attachment_path
            .cmp(&right.attachment_path)
            .then(left.source_path.cmp(&right.source_path))
    });
    hits
}

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
        .map(|value| value.trim().trim_start_matches('.').to_ascii_lowercase())
        .unwrap_or_default()
}

pub(crate) fn attachment_kind_label(kind: LinkGraphAttachmentKind) -> &'static str {
    match kind {
        LinkGraphAttachmentKind::Image => "image",
        LinkGraphAttachmentKind::Pdf => "pdf",
        LinkGraphAttachmentKind::Gpg => "gpg",
        LinkGraphAttachmentKind::Document => "document",
        LinkGraphAttachmentKind::Archive => "archive",
        LinkGraphAttachmentKind::Audio => "audio",
        LinkGraphAttachmentKind::Video => "video",
        LinkGraphAttachmentKind::Other => "other",
    }
}
