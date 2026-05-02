use super::{
    LinkGraphDocument, LinkGraphHit, LinkGraphIndex, LinkGraphSearchOptions, LinkGraphVirtualNode,
    PageIndexNode,
};
use std::collections::HashMap;
use std::path::Path;

impl LinkGraphIndex {
    /// Default threshold where delta refresh switches to full rebuild.
    #[must_use]
    pub const fn incremental_rebuild_threshold() -> usize {
        super::INCREMENTAL_REBUILD_THRESHOLD
    }

    /// Notebook root used by this index.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Directories included in this index (from configuration).
    #[must_use]
    pub fn include_dirs(&self) -> &[String] {
        &self.include_dirs
    }

    /// Iterate over all indexed documents.
    #[cfg(feature = "studio")]
    pub(crate) fn docs(&self) -> std::collections::hash_map::Values<'_, String, LinkGraphDocument> {
        self.docs_by_id.values()
    }

    #[allow(dead_code)]
    pub(in crate::link_graph::index) fn execute_direct_id_lookup(
        &self,
        direct_id: &str,
        _limit: usize,
        _options: &LinkGraphSearchOptions,
    ) -> Vec<LinkGraphHit> {
        let mut out = Vec::new();
        if let Some(doc_id) = self.resolve_doc_id(direct_id)
            && let Some(doc) = self.docs_by_id.get(doc_id)
        {
            out.push(LinkGraphHit {
                stem: doc.stem.clone(),
                title: doc.title.clone(),
                path: doc.path.clone(),
                doc_type: doc.doc_type.clone(),
                tags: doc.tags.clone(),
                score: 1.0,
                best_section: None,
                match_reason: Some("direct_id".to_string()),
            });
        }
        out
    }

    /// Resolve one document or anchor id into its semantic breadcrumb trail.
    #[must_use]
    pub fn page_index_semantic_path(&self, anchor_id: &str) -> Option<Vec<String>> {
        self.extract_lineage(anchor_id)
    }

    pub(crate) fn extract_lineage(&self, anchor_id: &str) -> Option<Vec<String>> {
        let anchor_id = self.canonical_anchor_id(anchor_id)?;
        let doc_id = canonical_doc_id(anchor_id.as_str());

        if anchor_id == doc_id {
            return self.root_lineage(doc_id);
        }

        let roots = self.get_tree(doc_id)?;
        let node_ids = collect_lineage_node_ids(self.get_node_parent_map(), anchor_id.as_str())?;
        node_ids
            .into_iter()
            .map(|node_id| find_node_title(roots, node_id.as_str()))
            .collect()
    }

    pub(crate) fn has_doc(&self, doc_id: &str) -> bool {
        self.docs_by_id.contains_key(doc_id)
    }

    pub(crate) fn get_doc(&self, doc_id: &str) -> Option<&LinkGraphDocument> {
        self.docs_by_id.get(doc_id)
    }

    pub(crate) fn get_tree(&self, doc_id: &str) -> Option<&Vec<PageIndexNode>> {
        self.trees_by_doc.get(doc_id)
    }

    pub(crate) fn get_node_parent_map(&self) -> &HashMap<String, Option<String>> {
        &self.node_parent_map
    }

    pub(crate) fn resolve_doc_id_pub(&self, stem_or_id: &str) -> Option<&str> {
        self.resolve_doc_id(stem_or_id)
    }

    fn canonical_anchor_id(&self, anchor_id: &str) -> Option<String> {
        let trimmed = anchor_id.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some((doc_ref, suffix)) = trimmed.split_once('#') {
            let doc_id = if self.has_doc(doc_ref) {
                doc_ref.to_string()
            } else {
                self.resolve_doc_id_pub(doc_ref)?.to_string()
            };
            let suffix = suffix.trim_matches(|ch: char| ch == '#' || ch.is_whitespace());
            return if suffix.is_empty() {
                Some(doc_id)
            } else {
                Some(format!("{doc_id}#{suffix}"))
            };
        }

        if self.has_doc(trimmed) {
            return Some(trimmed.to_string());
        }

        self.resolve_doc_id_pub(trimmed).map(str::to_string)
    }

    fn root_lineage(&self, doc_id: &str) -> Option<Vec<String>> {
        if let Some(roots) = self.get_tree(doc_id)
            && let Some(root) = roots.first()
        {
            return Some(vec![root.title.clone()]);
        }

        self.get_doc(doc_id).map(|doc| vec![doc.title.clone()])
    }

    /// Get document relative path by stem or ID.
    #[must_use]
    pub fn doc_path(&self, stem_or_id: &str) -> Option<&str> {
        let doc_id = self.resolve_doc_id(stem_or_id)?;
        self.docs_by_id.get(doc_id).map(|doc| doc.path.as_str())
    }

    /// Get document title by stem or ID.
    #[must_use]
    pub fn doc_title(&self, stem_or_id: &str) -> Option<&str> {
        let doc_id = self.resolve_doc_id(stem_or_id)?;
        self.docs_by_id.get(doc_id).map(|doc| doc.title.as_str())
    }

    /// Get all page index trees for Triple-A addressing.
    #[must_use]
    pub fn all_page_index_trees(&self) -> &HashMap<String, Vec<PageIndexNode>> {
        &self.trees_by_doc
    }

    /// Get all virtual nodes created by knowledge distillation.
    ///
    /// Virtual nodes represent collapsed dense clusters of high-saliency nodes.
    /// They inherit edges from their member nodes and can be used for graph traversal.
    #[must_use]
    pub fn virtual_nodes(&self) -> Vec<LinkGraphVirtualNode> {
        self.virtual_nodes
            .values()
            .map(|vn| LinkGraphVirtualNode {
                id: vn.id.clone(),
                members: vn.members.clone(),
                avg_saliency: vn.avg_saliency,
                title: vn.title.clone(),
                internal_edges: vn.internal_edges,
                edge_density: vn.edge_density,
            })
            .collect()
    }

    /// Extract semantic intent targets for a document.
    #[must_use]
    pub fn intent_targets(&self, doc_id: &str) -> (Vec<String>, Vec<String>) {
        let Some(doc) = self.docs_by_id.get(doc_id) else {
            return (Vec::new(), Vec::new());
        };
        // This is a simplification, actual implementation might need more parsing.
        (doc.tags.clone(), Vec::new())
    }

    /// Build a `RegistryIndex` for O(1) ID lookups.
    ///
    /// The registry index provides fast access to nodes with explicit `:ID:` attributes.
    #[must_use]
    pub fn build_registry_index(&self) -> crate::link_graph::addressing::RegistryIndex {
        crate::link_graph::addressing::RegistryIndex::build_from_trees(&self.trees_by_doc)
    }

    /// Build a `RegistryIndex` with collision detection.
    ///
    /// Returns both the registry index and any ID collisions detected.
    /// This is the recommended method for semantic validation.
    #[must_use]
    pub fn build_registry_index_with_collisions(
        &self,
    ) -> crate::link_graph::addressing::RegistryBuildResult {
        crate::link_graph::addressing::RegistryIndex::build_from_trees_with_collisions(
            &self.trees_by_doc,
        )
    }

    /// Build a `TopologyIndex` for fuzzy path discovery.
    ///
    /// The topology index enables structural path lookup and fuzzy matching.
    #[must_use]
    pub fn build_topology_index(&self) -> crate::link_graph::addressing::TopologyIndex {
        crate::link_graph::addressing::TopologyIndex::build_from_trees(&self.trees_by_doc)
    }
}

fn canonical_doc_id(anchor_id: &str) -> &str {
    anchor_id
        .split_once('#')
        .map_or(anchor_id, |(doc_id, _)| doc_id)
}

fn collect_lineage_node_ids(
    node_parent_map: &HashMap<String, Option<String>>,
    target_node_id: &str,
) -> Option<Vec<String>> {
    let mut node_ids = Vec::new();
    let mut current = Some(target_node_id.to_string());
    let mut visited = std::collections::HashSet::new();

    while let Some(node_id) = current {
        if !visited.insert(node_id.clone()) {
            return None;
        }
        let parent_id = node_parent_map.get(node_id.as_str())?.clone();
        node_ids.push(node_id);
        current = parent_id;
    }

    node_ids.reverse();
    Some(node_ids)
}

fn find_node_title(nodes: &[PageIndexNode], target_node_id: &str) -> Option<String> {
    for node in nodes {
        if node.node_id == target_node_id {
            return Some(node.title.clone());
        }
        if let Some(title) = find_node_title(&node.children, target_node_id) {
            return Some(title);
        }
    }
    None
}
