use super::super::attachments::{attachments_for_parsed_note, expand_pdf_attachments};
use crate::link_graph::index::{IndexedSection, LinkGraphIndex};
use crate::parsers::markdown::{ParsedNote, normalize_alias};
use std::collections::HashSet;

impl LinkGraphIndex {
    pub(super) fn rebuild_from_current_filters(&self) -> Result<Self, String> {
        Self::build_with_filters(&self.root, &self.include_dirs, &self.excluded_dirs)
    }

    pub(super) fn recompute_edge_count(&mut self) {
        self.edge_count = self.outgoing.values().map(HashSet::len).sum();
    }

    pub(super) fn recompute_rank_by_id(&mut self) {
        self.rank_by_id =
            Self::compute_rank_by_id(&self.docs_by_id, &self.incoming, &self.outgoing);
    }

    pub(super) fn prune_empty_edge_sets(&mut self) {
        self.outgoing.retain(|_, targets| !targets.is_empty());
        self.incoming.retain(|_, sources| !sources.is_empty());
    }

    pub(super) fn remove_doc_by_id(&mut self, doc_id: &str) {
        self.docs_by_id.remove(doc_id);
        self.remove_passages_for_doc(doc_id);
        self.remove_page_index_for_doc(doc_id);
        self.sections_by_doc.remove(doc_id);
        self.attachments_by_doc.remove(doc_id);
        self.alias_to_doc_id
            .retain(|_, existing| existing != doc_id);
        self.outgoing.remove(doc_id);
        self.incoming.remove(doc_id);
        for targets in self.outgoing.values_mut() {
            targets.remove(doc_id);
        }
        for sources in self.incoming.values_mut() {
            sources.remove(doc_id);
        }
        self.prune_empty_edge_sets();
    }

    pub(super) fn insert_doc_no_edges(&mut self, parsed: &ParsedNote) {
        let doc = &parsed.doc;
        self.docs_by_id.insert(doc.id.clone(), doc.clone());
        self.sections_by_doc.insert(
            doc.id.clone(),
            parsed
                .sections
                .iter()
                .map(IndexedSection::from_parsed)
                .collect::<Vec<IndexedSection>>(),
        );
        self.rebuild_passages_for_doc(&doc.id);
        self.rebuild_page_index_for_doc(&doc.id);
        let mut attachments = attachments_for_parsed_note(parsed);
        expand_pdf_attachments(&mut attachments);
        self.attachments_by_doc
            .insert(doc.id.clone(), attachments);
        for alias in [&doc.id, &doc.path, &doc.stem] {
            let key = normalize_alias(alias);
            if key.is_empty() {
                continue;
            }
            self.alias_to_doc_id.insert(key, doc.id.clone());
        }
    }

    pub(super) fn add_outgoing_links_for_doc(&mut self, parsed: &ParsedNote) {
        let from_id = parsed.doc.id.clone();
        for raw_target in &parsed.link_targets {
            let normalized = normalize_alias(raw_target);
            if normalized.is_empty() {
                continue;
            }
            let Some(to_id) = self.alias_to_doc_id.get(&normalized).cloned() else {
                continue;
            };
            if to_id == from_id {
                continue;
            }
            let inserted = self
                .outgoing
                .entry(from_id.clone())
                .or_default()
                .insert(to_id.clone());
            if inserted {
                self.incoming
                    .entry(to_id)
                    .or_default()
                    .insert(from_id.clone());
            }
        }
    }
}
