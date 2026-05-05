use crate::link_graph::index::{
    LinkGraphDocument, LinkGraphIndex, LinkGraphMetadata, doc_sort_key,
};
use crate::parsers::markdown::normalize_alias;

impl LinkGraphIndex {
    /// Get per-note metadata.
    #[must_use]
    pub fn metadata(&self, stem_or_id: &str) -> Option<LinkGraphMetadata> {
        let doc_id = self.resolve_doc_id(stem_or_id)?;
        let doc = self.docs_by_id.get(doc_id)?;
        Some(LinkGraphMetadata {
            stem: doc.stem.clone(),
            title: doc.title.clone(),
            path: doc.path.clone(),
            tags: doc.tags.clone(),
        })
    }

    /// Resolve ambiguous stem/id/path input into deterministic metadata candidates.
    #[must_use]
    pub fn resolve_metadata_candidates(&self, stem_or_id: &str) -> Vec<LinkGraphMetadata> {
        let key = normalize_alias(stem_or_id);
        if key.is_empty() {
            return Vec::new();
        }

        let mut out: Vec<LinkGraphMetadata> = self
            .docs_by_id
            .values()
            .filter(|doc| {
                normalize_alias(&doc.id) == key
                    || normalize_alias(&doc.path) == key
                    || normalize_alias(&doc.stem) == key
            })
            .map(|doc| LinkGraphMetadata {
                stem: doc.stem.clone(),
                title: doc.title.clone(),
                path: doc.path.clone(),
                tags: doc.tags.clone(),
            })
            .collect();
        out.sort_by(|left, right| left.path.cmp(&right.path));
        out
    }

    /// Return table-of-contents rows.
    #[must_use]
    pub fn toc(&self, limit: usize) -> Vec<LinkGraphDocument> {
        let mut docs: Vec<LinkGraphDocument> = self.docs_by_id.values().cloned().collect();
        docs.sort_by(|a, b| doc_sort_key(a).cmp(&doc_sort_key(b)));
        docs.truncate(limit.max(1));
        docs
    }
}
