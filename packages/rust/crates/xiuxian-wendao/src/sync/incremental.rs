//! Incremental filesystem discovery and manifest diffing.

use super::{DiscoveryOptions, SyncManifest, SyncResult};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Common extension policy for incremental sync routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalSyncPolicy {
    /// File extensions allowed for sync (e.g. `md`, `txt`).
    pub extensions: Vec<String>,
    /// Glob patterns used to include files.
    pub include_globs: Vec<String>,
    /// Glob patterns used to exclude files.
    pub exclude_globs: Vec<String>,
}

impl Default for IncrementalSyncPolicy {
    fn default() -> Self {
        Self {
            extensions: vec!["md".to_string(), "markdown".to_string()],
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
        }
    }
}

impl IncrementalSyncPolicy {
    /// Create a new policy with explicit extensions.
    #[must_use]
    pub fn new(extensions: &[String]) -> Self {
        Self {
            extensions: extensions.to_vec(),
            ..Self::default()
        }
    }

    /// Derives sync policy from glob patterns.
    #[must_use]
    pub fn from_glob_patterns(patterns: &[String], fallback_extensions: &[&str]) -> Self {
        let mut extensions = extract_extensions_from_glob_patterns(patterns);
        if extensions.is_empty() {
            extensions = fallback_extensions
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
        }
        Self {
            extensions,
            include_globs: patterns.to_vec(),
            ..Self::default()
        }
    }

    /// Returns true if the path extension matches policy.
    #[must_use]
    pub fn supports_path(&self, path: &Path) -> bool {
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            return false;
        };
        let lower = ext.to_lowercase();
        self.extensions.iter().any(|e| e == &lower)
    }
}

/// Helper to extract base extensions from a list of globs.
#[must_use]
pub fn extract_extensions_from_glob_patterns(patterns: &[String]) -> Vec<String> {
    let mut values = BTreeSet::new();
    for pattern in patterns {
        if let Some(index) = pattern.rfind("*.") {
            let ext = &pattern[index + 2..];
            values.insert(ext.to_lowercase());
        }
    }
    values.into_iter().collect()
}

/// Core synchronization engine.
#[derive(Debug, Clone)]
pub struct SyncEngine {
    /// Root directory of the project to sync.
    pub project_root: PathBuf,
    /// Path where sync manifest is persisted.
    pub manifest_path: PathBuf,
    /// Discovery behavior options.
    pub options: DiscoveryOptions,
}

impl SyncEngine {
    /// Construct a new sync engine for a project.
    #[must_use]
    pub fn new(project_root: PathBuf, manifest_path: PathBuf) -> Self {
        Self {
            project_root,
            manifest_path,
            options: DiscoveryOptions::default(),
        }
    }

    /// Attach discovery options to the engine.
    #[must_use]
    pub fn with_options(mut self, options: DiscoveryOptions) -> Self {
        self.options = options;
        self
    }

    /// Discover files under the project root according to discovery options.
    #[must_use]
    pub fn discover_files(&self) -> Vec<PathBuf> {
        let root = self.project_root.as_path();
        if !root.is_dir() {
            return Vec::new();
        }

        let filter = DiscoveryFilter::from_options(&self.options);
        let mut files = discover_files_under_root(root, &self.options, &filter);
        files.sort();
        files
    }

    /// Compute diff between a manifest snapshot and current file list.
    #[must_use]
    pub fn compute_diff(&self, manifest: &SyncManifest, files: &[PathBuf]) -> SyncResult {
        let mut result = SyncResult::default();
        let mut seen: HashSet<String> = HashSet::new();

        for file in files {
            let key = manifest_key_for_path(file, &self.project_root);
            seen.insert(key.clone());
            match manifest.0.get(&key) {
                None => result.added.push(file.clone()),
                Some(previous) => match Self::compute_file_hash(file) {
                    Some(current) if current == *previous => result.unchanged += 1,
                    _ => result.modified.push(file.clone()),
                },
            }
        }

        for key in manifest.0.keys() {
            if !seen.contains(key) {
                result.deleted.push(PathBuf::from(key));
            }
        }

        result
    }
}

struct DiscoveryFilter {
    extensions: HashSet<String>,
    skip_dirs: HashSet<String>,
}

impl DiscoveryFilter {
    fn from_options(options: &DiscoveryOptions) -> Self {
        Self {
            extensions: options
                .extensions
                .iter()
                .map(|ext| ext.to_ascii_lowercase())
                .collect(),
            skip_dirs: options
                .skip_dirs
                .iter()
                .map(|name| name.to_ascii_lowercase())
                .collect(),
        }
    }

    fn should_descend(&self, entry: &walkdir::DirEntry, options: &DiscoveryOptions) -> bool {
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            return true;
        }

        let name = entry.file_name().to_string_lossy();
        if options.skip_hidden && name.starts_with('.') {
            return false;
        }
        !self.skip_dirs.contains(name.to_ascii_lowercase().as_str())
    }

    fn accepts_file(&self, path: &Path, options: &DiscoveryOptions) -> bool {
        file_is_visible(path, options)
            && file_extension_is_supported(path, &self.extensions)
            && file_size_is_allowed(path, options.max_file_size)
    }
}

fn discover_files_under_root(
    root: &Path,
    options: &DiscoveryOptions,
    filter: &DiscoveryFilter,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| filter.should_descend(entry, options))
        .filter_map(Result::ok)
    {
        if accepted_file_entry(&entry, options, filter) {
            files.push(entry.path().to_path_buf());
        }
        if discovery_limit_reached(files.len(), options.max_files) {
            break;
        }
    }
    files
}

fn accepted_file_entry(
    entry: &walkdir::DirEntry,
    options: &DiscoveryOptions,
    filter: &DiscoveryFilter,
) -> bool {
    entry.file_type().is_file() && filter.accepts_file(entry.path(), options)
}

fn file_is_visible(path: &Path, options: &DiscoveryOptions) -> bool {
    !options.skip_hidden || !is_hidden_path(path)
}

fn file_extension_is_supported(path: &Path, extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| extensions.contains(&ext.to_ascii_lowercase()))
}

fn file_size_is_allowed(path: &Path, max_file_size: u64) -> bool {
    path.metadata()
        .map_or(true, |metadata| metadata.len() <= max_file_size)
}

fn discovery_limit_reached(file_count: usize, max_files: Option<usize>) -> bool {
    max_files.is_some_and(|limit| file_count >= limit)
}

fn is_hidden_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn manifest_key_for_path(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.to_string_lossy().replace('\\', "/")
}
