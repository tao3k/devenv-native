//! `dependency_indexer::symbols::index::types` owns Wendao symbols index types behavior.

use std::collections::HashMap;

use crate::dependency_indexer::symbols::ExternalSymbol;

/// Symbol index for fast lookup.
#[derive(Debug, Default, Clone)]
pub struct SymbolIndex {
    /// Symbols grouped by crate/package
    pub(super) by_crate: Vec<CrateSymbols>,
    /// Map `crate_name` -> index
    pub(super) crate_map: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub(super) struct CrateSymbols {
    pub(super) name: String,
    pub(super) symbols: Vec<ExternalSymbol>,
}

impl SymbolIndex {
    /// Create an empty symbol index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_crate: Vec::new(),
            crate_map: HashMap::new(),
        }
    }
}
