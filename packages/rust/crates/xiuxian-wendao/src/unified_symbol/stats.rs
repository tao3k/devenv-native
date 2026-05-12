//! `unified_symbol::stats` owns Wendao unified symbol stats behavior.

/// Statistics for unified symbol index.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct UnifiedIndexStats {
    /// Total number of indexed symbols.
    pub total_symbols: usize,
    /// Number of project-local symbols.
    pub project_symbols: usize,
    /// Number of external dependency symbols.
    pub external_symbols: usize,
    /// Number of external crates with recorded usage.
    pub external_crates: usize,
    /// Number of project files that reference external symbols.
    pub project_files_with_externals: usize,
}
