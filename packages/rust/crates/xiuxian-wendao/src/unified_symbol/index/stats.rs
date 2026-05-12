use crate::unified_symbol::{UnifiedIndexStats, UnifiedSymbolIndex};

impl UnifiedSymbolIndex {
    /// Return index statistics.
    #[must_use]
    pub fn stats(&self) -> UnifiedIndexStats {
        let project_symbols = self
            .symbols
            .iter()
            .filter(|symbol| symbol.is_project())
            .count();
        let external_symbols = self.symbols.len().saturating_sub(project_symbols);

        UnifiedIndexStats {
            total_symbols: self.symbols.len(),
            project_symbols,
            external_symbols,
            external_crates: self.external_usage.len(),
            project_files_with_externals: self.project_files.len(),
        }
    }
}
