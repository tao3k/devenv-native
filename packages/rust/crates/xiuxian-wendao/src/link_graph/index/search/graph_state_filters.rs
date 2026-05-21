use crate::link_graph::index::{
    LinkGraphDocument, LinkGraphIndex, LinkGraphSearchOptions, doc_contains_phrase,
};
use std::collections::HashSet;

impl LinkGraphIndex {
    fn has_missing_backlink(&self, doc_id: &str) -> bool {
        let Some(targets) = self.outgoing.get(doc_id) else {
            return false;
        };
        targets.iter().any(|target| {
            !self
                .outgoing
                .get(target)
                .is_some_and(|row| row.contains(doc_id))
        })
    }

    pub(super) fn matches_graph_state_filters(
        &self,
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
        mention_filters: &[String],
    ) -> bool {
        if options.filters.orphan {
            let outgoing_empty = self.outgoing.get(&doc.id).is_none_or(HashSet::is_empty);
            let incoming_empty = self.incoming.get(&doc.id).is_none_or(HashSet::is_empty);
            if !outgoing_empty || !incoming_empty {
                return false;
            }
        }

        if options.filters.tagless && !doc.tags.is_empty() {
            return false;
        }

        if options.filters.missing_backlink && !self.has_missing_backlink(&doc.id) {
            return false;
        }

        if !mention_filters.is_empty()
            && !mention_filters
                .iter()
                .any(|phrase| doc_contains_phrase(doc, phrase, options.case_sensitive))
        {
            return false;
        }

        true
    }
}
