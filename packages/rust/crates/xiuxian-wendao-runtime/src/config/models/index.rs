//! Index runtime configuration records for Wendao search scope.

/// Resolved index-scope directory filters for the link graph.
#[derive(Debug, Clone, Default)]
pub struct LinkGraphIndexRuntimeConfig {
    /// Relative include directories used for indexing scope.
    pub include_dirs: Vec<String>,
    /// Relative directories excluded from indexing scope.
    pub exclude_dirs: Vec<String>,
}
