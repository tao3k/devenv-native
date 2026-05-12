//! `sync::types` owns Wendao sync types behavior.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Manifest entry: file path -> content hash
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SyncManifest(pub HashMap<String, String>);

/// File change type
#[derive(Debug, Clone, PartialEq)]
pub enum FileChange {
    /// A new file was added.
    Added(PathBuf),
    /// An existing file was modified.
    Modified(PathBuf),
    /// A tracked file was deleted.
    Deleted(PathBuf),
}

/// Sync result
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Newly added files.
    pub added: Vec<PathBuf>,
    /// Modified files.
    pub modified: Vec<PathBuf>,
    /// Deleted files.
    pub deleted: Vec<PathBuf>,
    /// Number of unchanged files.
    pub unchanged: usize,
}

/// File discovery options
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    /// Optional maximum number of files to scan.
    pub max_files: Option<usize>,
    /// Whether to skip hidden files and folders.
    pub skip_hidden: bool,
    /// Directory names to skip during discovery.
    pub skip_dirs: Vec<String>,
    /// Maximum file size in bytes.
    pub max_file_size: u64,
    /// Allowed file extensions.
    pub extensions: Vec<String>,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            max_files: None,
            skip_hidden: true,
            skip_dirs: vec![
                ".git".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "__pycache__".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".cache".to_string(),
            ],
            max_file_size: 1024 * 1024, // 1MB
            extensions: vec![
                "py".to_string(),
                "rs".to_string(),
                "md".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "json".to_string(),
                "toml".to_string(),
            ],
        }
    }
}
