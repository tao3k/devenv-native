//! `dependency_indexer::indexer::types` owns Wendao dependency indexer indexer types behavior.

use serde::{Deserialize, Serialize};

/// Namespace boundary: this public name is scoped by its module owner.
/// Dependency indexer configuration (for Python bindings compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    /// Project root path used for manifest discovery.
    pub project_root: String,
    /// Optional path to dependency index config file.
    pub config_path: Option<String>,
}

impl Default for DependencyConfig {
    fn default() -> Self {
        Self {
            project_root: ".".to_string(),
            config_path: None,
        }
    }
}

/// Result of dependency indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyIndexResult {
    /// Number of manifests processed.
    pub files_processed: usize,
    /// Number of extracted symbols.
    pub total_symbols: usize,
    /// Number of failed manifest/crate processing operations.
    pub errors: usize,
    /// Number of crates successfully indexed.
    pub crates_indexed: usize,
    /// Detailed error messages for failed crate processing
    pub error_details: Vec<String>,
}

/// Statistics about the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStats {
    /// Number of indexed crates.
    pub total_crates: usize,
    /// Number of indexed symbols.
    pub total_symbols: usize,
}
