use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::link_graph::index::build::assemble::types::NoteTables;
use crate::link_graph::index::{IndexedSection, doc_sort_key};
use crate::link_graph::models::LinkGraphAttachment;
use crate::parsers::markdown::{ParsedNote, normalize_alias, parse_note};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub(crate) fn parse_notes(root: &Path, candidate_paths: Vec<PathBuf>) -> Vec<ParsedNote> {
    let mut parsed_notes: Vec<ParsedNote> = candidate_paths
        .into_par_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(&path).ok()?;
            parse_note(&path, root, &content)
        })
        .collect();
    parsed_notes.sort_by(|left, right| doc_sort_key(&left.doc).cmp(&doc_sort_key(&right.doc)));
    parsed_notes
}

pub(crate) fn build_note_tables(parsed_notes: Vec<ParsedNote>) -> NoteTables {
    let mut docs_by_id = HashMap::new();
    let mut sections_by_doc = HashMap::new();
    let mut attachments_by_doc = HashMap::new();
    let mut alias_to_doc_id = HashMap::new();

    for parsed in &parsed_notes {
        let doc = &parsed.doc;
        docs_by_id.insert(doc.id.clone(), doc.clone());
        sections_by_doc.insert(
            doc.id.clone(),
            parsed
                .sections
                .iter()
                .map(IndexedSection::from_parsed)
                .collect(),
        );
        attachments_by_doc.insert(doc.id.clone(), attachments_for_parsed_note(parsed));
        for alias in [&doc.id, &doc.path, &doc.stem] {
            let key = normalize_alias(alias);
            if key.is_empty() {
                continue;
            }
            alias_to_doc_id.entry(key).or_insert_with(|| doc.id.clone());
        }
    }

    // Expand Docling-supported document attachments into derived resources.
    for attachments in attachments_by_doc.values_mut() {
        crate::link_graph::index::build::attachments::expand_document_attachments(attachments);
    }

    NoteTables {
        parsed_notes,
        docs_by_id,
        sections_by_doc,
        attachments_by_doc,
        alias_to_doc_id,
    }
}

fn attachments_for_parsed_note(parsed: &ParsedNote) -> Vec<LinkGraphAttachment> {
    crate::link_graph::index::build::attachments::attachments_for_parsed_note(parsed)
}
