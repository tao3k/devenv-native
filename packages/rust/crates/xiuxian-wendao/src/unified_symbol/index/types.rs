//! `unified_symbol::index::types` owns Wendao unified symbol index types behavior.

use crate::search::SearchDocumentIndex;
use crate::unified_symbol::UnifiedSymbol;
use crate::unified_symbol::symbol::SymbolSource;
use std::collections::HashMap;

/// Unified Symbol Index - combines project and external symbols.
pub struct UnifiedSymbolIndex {
    /// All symbols indexed by lowercase name (Legacy in-memory path)
    pub(crate) by_name: HashMap<String, Vec<usize>>,
    /// All symbols stored in a vector
    pub(crate) symbols: Vec<UnifiedSymbol>,
    /// External crate usage in project (`crate_name` -> project locations)
    pub(crate) external_usage: HashMap<String, Vec<String>>,
    /// Project files that use external crates
    pub(crate) project_files: HashMap<String, Vec<String>>,

    // Shared search integration
    pub(crate) search_index: SearchDocumentIndex,
}

impl UnifiedSymbolIndex {
    /// Create an empty unified index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            symbols: Vec::new(),
            external_usage: HashMap::new(),
            project_files: HashMap::new(),
            search_index: SearchDocumentIndex::new(),
        }
    }

    /// Returns list of all unique external crate names in the index.
    #[must_use]
    pub fn get_external_crates(&self) -> Vec<String> {
        let mut crates = self
            .symbols
            .iter()
            .filter_map(|symbol| match &symbol.source {
                SymbolSource::External(name) => Some(name.clone()),
                SymbolSource::Project => None,
            })
            .collect::<Vec<_>>();
        crates.sort();
        crates.dedup();
        crates
    }

    /// Returns list of all unique project-local crate names in the index.
    #[must_use]
    pub fn get_project_crates(&self) -> Vec<String> {
        let mut crates = self
            .symbols
            .iter()
            .filter(|symbol| symbol.source == SymbolSource::Project)
            .map(|symbol| symbol.crate_name.clone())
            .collect::<Vec<_>>();
        crates.sort();
        crates.dedup();
        crates
    }
}

impl Default for UnifiedSymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UnifiedSymbolIndex {
    fn clone(&self) -> Self {
        Self {
            by_name: self.by_name.clone(),
            symbols: self.symbols.clone(),
            external_usage: self.external_usage.clone(),
            project_files: self.project_files.clone(),
            search_index: self.search_index.clone(),
        }
    }
}

impl std::fmt::Debug for UnifiedSymbolIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedSymbolIndex")
            .field("symbol_count", &self.symbols.len())
            .finish_non_exhaustive()
    }
}
