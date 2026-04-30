use super::content::extract_saliency_params;
use super::links::extract_link_targets_from_occurrences;
use super::paths::{is_org_note, normalize_slashes, relative_doc_id};
use super::sections::adapt_sections;
use super::time::resolve_note_timestamps;
use super::types::ParsedNote;
use crate::link_graph::LinkGraphDocument;
use std::path::Path;
use xiuxian_wendao_parsers::note::NoteCore;
use xiuxian_wendao_parsers::note::{MarkdownNote, MarkdownNoteCore, parse_markdown_note};
use xiuxian_wendao_parsers::org::{OrgNote, parse_org_note};

#[must_use]
pub(crate) fn adapt_markdown_note(
    path: &Path,
    root: &Path,
    parser_note: MarkdownNote,
) -> Option<ParsedNote> {
    let doc_id = relative_doc_id(path, root)?;
    let stem = path.file_stem()?.to_string_lossy().to_string();
    if stem.is_empty() {
        return None;
    }
    let rel_path = normalize_slashes(
        path.strip_prefix(root)
            .ok()
            .map_or_else(
                || path.to_string_lossy().to_string(),
                |p| p.to_string_lossy().to_string(),
            )
            .as_str(),
    );
    let parsed_document = parser_note.document;
    let frontmatter = parsed_document.raw_metadata;
    let core = parsed_document.core;
    let (saliency_base, decay_rate) = extract_saliency_params(frontmatter.as_ref());
    let (created_ts, modified_ts) = resolve_note_timestamps(frontmatter.as_ref(), path);
    let search_text = core.body;
    let search_text_lower = search_text.to_lowercase();
    let id_lower = doc_id.to_lowercase();
    let stem_lower = stem.to_lowercase();
    let path_lower = rel_path.to_lowercase();
    let title_lower = core.title.to_lowercase();
    let tags_lower: Vec<String> = core.tags.iter().map(|tag| tag.to_lowercase()).collect();
    let MarkdownNoteCore {
        references: _,
        targets,
        sections,
    } = parser_note.core;
    let extracted = extract_link_targets_from_occurrences(&targets, path, root);
    let sections = adapt_sections(sections, &targets, path, root);
    Some(ParsedNote {
        doc: LinkGraphDocument {
            id: doc_id,
            id_lower,
            stem,
            stem_lower,
            path: rel_path,
            path_lower,
            title: core.title,
            title_lower,
            tags: core.tags,
            tags_lower,
            lead: core.lead,
            doc_type: core.doc_type,
            word_count: core.word_count,
            search_text,
            search_text_lower,
            saliency_base,
            decay_rate,
            created_ts,
            modified_ts,
        },
        link_targets: extracted.note_links,
        attachment_targets: extracted.attachments,
        sections,
    })
}

#[must_use]
pub(crate) fn adapt_org_note(path: &Path, root: &Path, parser_note: OrgNote) -> Option<ParsedNote> {
    let doc_id = relative_doc_id(path, root)?;
    let stem = path.file_stem()?.to_string_lossy().to_string();
    if stem.is_empty() {
        return None;
    }
    let rel_path = normalize_slashes(
        path.strip_prefix(root)
            .ok()
            .map_or_else(
                || path.to_string_lossy().to_string(),
                |p| p.to_string_lossy().to_string(),
            )
            .as_str(),
    );
    let core = parser_note.document.core;
    let (saliency_base, decay_rate) = extract_saliency_params(None);
    let (created_ts, modified_ts) = resolve_note_timestamps(None, path);
    let search_text = core.body;
    let search_text_lower = search_text.to_lowercase();
    let id_lower = doc_id.to_lowercase();
    let stem_lower = stem.to_lowercase();
    let path_lower = rel_path.to_lowercase();
    let title_lower = core.title.to_lowercase();
    let tags_lower: Vec<String> = core.tags.iter().map(|tag| tag.to_lowercase()).collect();
    let NoteCore {
        references: _,
        targets,
        sections,
    } = parser_note.core;
    let extracted = extract_link_targets_from_occurrences(&targets, path, root);
    let sections = adapt_sections(sections, &targets, path, root);
    Some(ParsedNote {
        doc: LinkGraphDocument {
            id: doc_id,
            id_lower,
            stem,
            stem_lower,
            path: rel_path,
            path_lower,
            title: core.title,
            title_lower,
            tags: core.tags,
            tags_lower,
            lead: core.lead,
            doc_type: core.doc_type,
            word_count: core.word_count,
            search_text,
            search_text_lower,
            saliency_base,
            decay_rate,
            created_ts,
            modified_ts,
        },
        link_targets: extracted.note_links,
        attachment_targets: extracted.attachments,
        sections,
    })
}

/// Parse one note file into structured document row plus outgoing link targets.
#[must_use]
pub fn parse_note(path: &Path, root: &Path, content: &str) -> Option<ParsedNote> {
    let stem = path.file_stem()?.to_string_lossy().to_string();
    if stem.is_empty() {
        return None;
    }
    if is_org_note(path) {
        return adapt_org_note(path, root, parse_org_note(content, &stem));
    }
    adapt_markdown_note(path, root, parse_markdown_note(content, &stem))
}
