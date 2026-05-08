use super::content::extract_saliency_params;
use super::links::extract_link_targets_from_occurrences;
use super::paths::{is_org_note, normalize_slashes, relative_doc_id};
use super::sections::adapt_sections;
use super::time::resolve_note_timestamps;
use super::types::ParsedNote;
use crate::link_graph::LinkGraphDocument;
use serde_yaml::Value;
use std::collections::BTreeSet;
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
    let search_text = markdown_search_text(&core.body, frontmatter.as_ref(), &core.title);
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

fn markdown_search_text(body: &str, frontmatter: Option<&Value>, title: &str) -> String {
    let Some(id) = frontmatter
        .and_then(|metadata| metadata.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return body.to_string();
    };

    let mut search_text = body.to_string();
    if !search_text.ends_with('\n') {
        search_text.push('\n');
    }
    search_text.push_str("\nsemantic_id: ");
    search_text.push_str(id);
    search_text.push_str("\nsemantic_search_key: ");
    search_text.push_str(id);
    if !title.trim().is_empty() {
        search_text.push(' ');
        search_text.push_str(title.trim());
    }
    search_text.push('\n');
    for relation in semantic_relation_search_lines(frontmatter, id, title) {
        search_text.push_str(&relation);
        search_text.push('\n');
    }
    search_text
}

fn semantic_relation_search_lines(
    frontmatter: Option<&Value>,
    source_id: &str,
    title: &str,
) -> Vec<String> {
    let Some(relations) = frontmatter
        .and_then(|metadata| metadata.get("relations"))
        .and_then(Value::as_sequence)
    else {
        return Vec::new();
    };

    let mut lines = BTreeSet::new();
    for relation in relations {
        let Some(kind) = relation
            .get("kind")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|kind| !kind.is_empty())
        else {
            continue;
        };
        let Some(target) = relation
            .get("target")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|target| !target.is_empty())
        else {
            continue;
        };
        lines.insert(format!("semantic_relation: {kind} {target}"));
        lines.insert(format!(
            "semantic_relation_key: {source_id} {kind} {target}"
        ));
        if !title.trim().is_empty() {
            lines.insert(format!(
                "semantic_relation_search_key: {source_id} {} {kind} {target}",
                title.trim()
            ));
        }
    }
    lines.into_iter().collect()
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
