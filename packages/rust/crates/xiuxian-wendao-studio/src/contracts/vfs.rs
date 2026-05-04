use serde::{Deserialize, Serialize};
use specta::Type;

/// A single entry in the VFS (file or directory).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VfsEntry {
    /// Full path relative to the VFS root.
    pub path: String,
    /// File or directory name.
    pub name: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
    /// Last modified timestamp (Unix seconds).
    pub modified: u64,
    /// MIME content type guess for files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Project grouping label for multi-root monorepo views.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Root label under the grouped project node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// Configured project root used to resolve this VFS root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
    /// Configured project directories associated with the resolved root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_dirs: Option<Vec<String>>,
}

/// Category classification for VFS entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum VfsCategory {
    /// Directory/folder.
    Folder,
    /// Skill definition file.
    Skill,
    /// Documentation file.
    Doc,
    /// Knowledge base file.
    Knowledge,
    /// Other/uncategorized file.
    Other,
}

/// A scanned entry with metadata for VFS tree display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VfsScanEntry {
    /// Full path relative to the VFS root.
    pub path: String,
    /// File or directory name.
    pub name: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// Category classification for UI styling.
    pub category: VfsCategory,
    /// File size in bytes (0 for directories).
    pub size: u64,
    /// Last modified timestamp (Unix seconds).
    pub modified: u64,
    /// MIME content type guess for files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Whether the file has YAML frontmatter.
    pub has_frontmatter: bool,
    /// Wendao document ID if indexed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wendao_id: Option<String>,
    /// Project grouping label for multi-root monorepo views.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Root label under the grouped project node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// Configured project root used to resolve this VFS root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
    /// Configured project directories associated with the resolved root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_dirs: Option<Vec<String>>,
}

/// Result of a VFS scan operation.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VfsScanResult {
    /// All entries found during the scan.
    pub entries: Vec<VfsScanEntry>,
    /// Total number of files scanned.
    pub file_count: usize,
    /// Total number of directories scanned.
    pub dir_count: usize,
    /// Time taken for the scan in milliseconds.
    pub scan_duration_ms: u64,
}

/// Payload for file content operations.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VfsContentResponse {
    /// Full path to the file.
    pub path: String,
    /// MIME content type.
    pub content_type: String,
    /// Raw file content.
    pub content: String,
    /// File modification timestamp.
    pub modified: u64,
}
