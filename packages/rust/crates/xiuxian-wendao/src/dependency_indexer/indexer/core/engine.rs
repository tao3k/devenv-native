//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `dependency_indexer::indexer::core::indexer` owns Wendao indexer core indexer behavior.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::dependency_indexer::indexer::SymbolIndex;

/// Dependency indexer that coordinates parsing, fetching, and symbol extraction.
#[derive(Debug, Clone)]
pub struct DependencyIndexer {
    /// Symbol index for fast lookup
    pub symbol_index: SymbolIndex,
    /// Crate name to version mapping
    pub(super) crate_versions: HashMap<String, String>,
    /// Project root directory
    pub(super) project_root: PathBuf,
    /// Config path
    pub(super) config_path: Option<PathBuf>,
}

impl DependencyIndexer {
    /// Create a new dependency indexer.
    pub fn new(project_root: &str, config_path: Option<&str>) -> Self {
        Self {
            symbol_index: SymbolIndex::new(),
            crate_versions: HashMap::new(),
            project_root: PathBuf::from(project_root),
            config_path: config_path.map(PathBuf::from),
        }
    }
}
