use crate::link_graph::index::{LinkGraphIndex, LinkGraphStats};

impl LinkGraphIndex {
    /// Return normalized stats payload.
    #[must_use]
    pub fn stats(&self) -> LinkGraphStats {
        let total_notes = self.docs_by_id.len();
        let orphans = self
            .docs_by_id
            .keys()
            .filter(|doc_id| {
                let out_empty = self
                    .outgoing
                    .get(*doc_id)
                    .is_none_or(std::collections::HashSet::is_empty);
                let in_empty = self
                    .incoming
                    .get(*doc_id)
                    .is_none_or(std::collections::HashSet::is_empty);
                out_empty && in_empty
            })
            .count();
        LinkGraphStats {
            total_notes,
            orphans,
            links_in_graph: self.edge_count,
            nodes_in_graph: total_notes,
        }
    }
}
