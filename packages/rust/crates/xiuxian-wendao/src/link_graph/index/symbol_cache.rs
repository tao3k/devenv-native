use super::{LinkGraphIndex, SymbolCacheStats, SymbolRef};
use std::collections::HashSet;

impl LinkGraphIndex {
    // =========================================================================
    // Symbol-to-Node Inverted Index (Phase 6.3/6.4)
    // =========================================================================

    /// Look up documents containing a specific code symbol.
    ///
    /// This is the O(1) lookup for semantic change propagation.
    /// Given a symbol name (e.g., "`process_data`"), returns all documents
    /// with `:OBSERVE:` patterns that reference this symbol.
    #[must_use]
    pub fn lookup_symbol(&self, symbol: &str) -> Option<&[SymbolRef]> {
        self.symbol_to_docs.get(symbol).map(Vec::as_slice)
    }

    /// Get all symbols in the inverted index.
    pub fn all_symbols(&self) -> impl Iterator<Item = &String> {
        self.symbol_to_docs.keys()
    }

    /// Get the total number of indexed symbols.
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbol_to_docs.len()
    }

    /// Check if any symbols are indexed.
    #[must_use]
    pub fn has_symbols(&self) -> bool {
        !self.symbol_to_docs.is_empty()
    }

    // =========================================================================
    // Phase 6.5: Incremental Symbol Cache Updates
    // =========================================================================

    /// Refresh the symbol cache for a single document.
    ///
    /// Call this when a document's `:OBSERVE:` patterns may have changed.
    /// This performs a targeted update without rebuilding the entire index.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn refresh_symbol_cache_for_doc(&mut self, doc_id: &str) {
        // First, remove existing entries for this document.
        self.remove_symbol_refs_for_doc(doc_id);

        // Clone the tree to avoid borrow issues.
        let tree_clone = self.trees_by_doc.get(doc_id).cloned();

        // Then, re-index if the document has a page index tree.
        if let Some(tree) = tree_clone {
            self.index_symbols_from_tree_cloned(doc_id, &tree);
        }
    }

    /// Remove all symbol references for a document from the cache.
    fn remove_symbol_refs_for_doc(&mut self, doc_id: &str) {
        for refs in self.symbol_to_docs.values_mut() {
            refs.retain(|r| r.doc_id != doc_id);
        }
        // Clean up empty symbol entries.
        self.symbol_to_docs.retain(|_, refs| !refs.is_empty());
    }

    /// Index symbols from a cloned page index tree.
    fn index_symbols_from_tree_cloned(&mut self, doc_id: &str, nodes: &[super::PageIndexNode]) {
        use super::pattern_symbols::extract_pattern_symbols;

        for node in nodes {
            for obs in &node.metadata.observations {
                let symbols = extract_pattern_symbols(&obs.pattern);
                for symbol in symbols {
                    let symbol_ref = SymbolRef {
                        doc_id: doc_id.to_string(),
                        node_id: node.node_id.clone(),
                        pattern: obs.pattern.clone(),
                        language: obs.language.clone(),
                        line_number: obs.line_number,
                        scope: obs.scope.clone(),
                    };
                    self.symbol_to_docs
                        .entry(symbol)
                        .or_default()
                        .push(symbol_ref);
                }
            }
            // Recurse into children.
            self.index_symbols_from_tree_cloned(doc_id, &node.children);
        }
    }

    /// Get statistics about the symbol cache.
    #[must_use]
    pub fn symbol_cache_stats(&self) -> SymbolCacheStats {
        let total_refs: usize = self.symbol_to_docs.values().map(std::vec::Vec::len).sum();
        SymbolCacheStats {
            unique_symbols: self.symbol_to_docs.len(),
            total_references: total_refs,
        }
    }

    /// Check if a document has any indexed symbols.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn doc_has_symbols(&self, doc_id: &str) -> bool {
        self.symbol_to_docs
            .values()
            .any(|refs| refs.iter().any(|r| r.doc_id == doc_id))
    }

    /// Get all documents that have indexed symbols.
    #[must_use]
    pub fn docs_with_symbols(&self) -> Vec<&str> {
        let mut doc_ids: HashSet<&str> = HashSet::new();
        for refs in self.symbol_to_docs.values() {
            for r in refs {
                doc_ids.insert(&r.doc_id);
            }
        }
        doc_ids.into_iter().collect()
    }
}
