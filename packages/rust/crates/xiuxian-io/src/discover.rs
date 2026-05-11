//! High-performance file discovery for knowledge indexing.
//!
//! Uses standard library for walking, optimized for speed.

use std::fs;
use std::path::{Component, Path, PathBuf};

/// Options for file discovery.
#[derive(Debug, Clone)]
pub struct DiscoverOptions {
    /// File extensions to include (e.g., `[".py", ".md"]`)
    pub extensions: Vec<String>,
    /// Maximum file size in bytes (default: 1MB)
    pub max_file_size: u64,
    /// Skip hidden files and directories
    pub skip_hidden: bool,
    /// Skip directories by name (e.g., `["target", "node_modules"]`)
    pub skip_dirs: Vec<String>,
    /// Maximum files to return (None for unlimited)
    pub max_files: Option<usize>,
    /// Include subdirectories recursively
    pub recursive: bool,
}

impl Default for DiscoverOptions {
    fn default() -> Self {
        Self {
            extensions: vec![".py".to_string(), ".md".to_string()],
            max_file_size: 1024 * 1024, // 1MB
            skip_hidden: true,
            skip_dirs: vec!["target".to_string(), "node_modules".to_string()],
            max_files: None,
            recursive: true,
        }
    }
}

/// Discover files recursively in a directory matching extensions.
#[must_use]
pub fn discover_files(root: &str, options: &DiscoverOptions) -> Vec<String> {
    let root_path = PathBuf::from(root);

    if !root_path.exists() || !root_path.is_dir() {
        return Vec::new();
    }

    let extensions: Vec<String> = options
        .extensions
        .iter()
        .map(|e| normalize_extension(e))
        .collect();

    let skip_dirs_set: Vec<String> = options.skip_dirs.clone();

    let mut files: Vec<PathBuf> = Vec::new();

    if options.recursive {
        walk_directory(
            &root_path,
            &root_path,
            &extensions,
            options.max_file_size,
            options.skip_hidden,
            &skip_dirs_set,
            &mut files,
        );
    } else {
        read_directory(
            &root_path,
            &root_path,
            &extensions,
            options.max_file_size,
            options.skip_hidden,
            &mut files,
        );
    }

    // Apply max_files limit
    if let Some(max) = options.max_files
        && files.len() > max
    {
        files.truncate(max);
    }

    // Sort and convert to relative paths
    files.sort();
    files
        .into_iter()
        .filter_map(|p| p.to_str().map(ToString::to_string))
        .collect()
}

/// Walk directory recursively.
fn walk_directory(
    root: &PathBuf,
    current: &PathBuf,
    extensions: &[String],
    max_file_size: u64,
    skip_hidden: bool,
    skip_dirs: &[String],
    files: &mut Vec<PathBuf>,
) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip hidden directories
            if skip_hidden
                && let Some(name) = path.file_name()
                && let Some(s) = name.to_str()
                && s.starts_with('.')
            {
                continue;
            }

            if path.is_dir() {
                // Check if this directory should be skipped
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if skip_dirs.iter().any(|skip| *skip == dir_name) {
                    continue;
                }

                // Recurse into subdirectory
                walk_directory(
                    root,
                    &path,
                    extensions,
                    max_file_size,
                    skip_hidden,
                    skip_dirs,
                    files,
                );
            } else if path.is_file() {
                // Check extension - path.extension() returns "py", we need ".py" to match
                if let Some(ext_os) = path.extension() {
                    let ext = ext_os.to_string_lossy().to_lowercase();
                    let ext_with_dot = format!(".{ext}");
                    if extensions.contains(&ext_with_dot) || extensions.contains(&ext) {
                        // Check file size
                        if let Ok(metadata) = entry.metadata()
                            && metadata.len() <= max_file_size
                            && let Ok(rel_path) = path.strip_prefix(root)
                        {
                            files.push(rel_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }
}

/// Read a single directory (non-recursive).
fn read_directory(
    root: &PathBuf,
    current: &PathBuf,
    extensions: &[String],
    max_file_size: u64,
    skip_hidden: bool,
    files: &mut Vec<PathBuf>,
) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                // Skip hidden files
                if skip_hidden
                    && let Some(name) = path.file_name()
                    && let Some(s) = name.to_str()
                    && s.starts_with('.')
                {
                    continue;
                }

                // Check extension - path.extension() returns "py", we need ".py" to match
                if let Some(ext_os) = path.extension() {
                    let ext = ext_os.to_string_lossy().to_lowercase();
                    let ext_with_dot = format!(".{ext}");
                    if extensions.contains(&ext_with_dot) || extensions.contains(&ext) {
                        // Check file size
                        if let Ok(metadata) = entry.metadata()
                            && metadata.len() <= max_file_size
                            && let Ok(rel_path) = path.strip_prefix(root)
                        {
                            files.push(rel_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }
}

/// Discover files in a single directory (non-recursive).
#[must_use]
pub fn discover_files_in_dir(
    dir: &str,
    extensions: &[String],
    max_file_size: u64,
    skip_hidden: bool,
) -> Vec<String> {
    discover_files_in_dir_impl(dir, extensions, max_file_size, skip_hidden)
}

fn discover_files_in_dir_impl(
    dir: &str,
    extensions: &[String],
    max_file_size: u64,
    skip_hidden: bool,
) -> Vec<String> {
    let dir_path = PathBuf::from(dir);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Vec::new();
    }

    // Normalize extensions to have dot prefix for matching
    let extensions: Vec<String> = extensions.iter().map(|e| normalize_extension(e)).collect();

    let mut files = read_dir_paths(&dir_path, &extensions, max_file_size, skip_hidden);

    files.sort();
    files
        .into_iter()
        .filter_map(|p| p.to_str().map(ToString::to_string))
        .collect()
}

/// Count files matching extensions in a directory.
#[must_use]
pub fn count_files_in_dir(dir: &str, extensions: &[String], skip_hidden: bool) -> usize {
    discover_files_in_dir(dir, extensions, u64::MAX, skip_hidden).len()
}

/// Check if a path should be skipped.
#[must_use]
pub fn should_skip_path(path: &str, skip_hidden: bool, skip_dirs: &[String]) -> bool {
    should_skip_path_impl(path, skip_hidden, skip_dirs)
}

fn should_skip_path_impl(path: &str, skip_hidden: bool, skip_dirs: &[String]) -> bool {
    let path = PathBuf::from(path);
    has_skipped_component(&path, skip_dirs) || has_hidden_file_name(&path, skip_hidden)
}

/// Normalize extension to lowercase with dot prefix.
fn normalize_extension(ext: &str) -> String {
    let mut ext = ext.to_lowercase();
    if !ext.starts_with('.') {
        ext = format!(".{ext}");
    }
    ext
}

fn read_dir_paths(
    dir_path: &Path,
    extensions: &[String],
    max_file_size: u64,
    skip_hidden: bool,
) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir_path) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            path_matches_file_discovery(&path, extensions, max_file_size, skip_hidden, &entry)
                .then_some(path)
        })
        .collect()
}

fn path_matches_file_discovery(
    path: &Path,
    extensions: &[String],
    max_file_size: u64,
    skip_hidden: bool,
    entry: &fs::DirEntry,
) -> bool {
    path.is_file()
        && !has_hidden_file_name(path, skip_hidden)
        && path_matches_extensions(path, extensions)
        && entry
            .metadata()
            .is_ok_and(|metadata| metadata.len() <= max_file_size)
}

fn path_matches_extensions(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .map(|ext_os| ext_os.to_string_lossy().to_lowercase())
        .is_some_and(|ext| extensions.contains(&format!(".{ext}")) || extensions.contains(&ext))
}

fn has_skipped_component(path: &Path, skip_dirs: &[String]) -> bool {
    path.components().any(|component| {
        matches!(component, Component::Normal(name) if name
            .to_str()
            .is_some_and(|name| skip_dirs.iter().any(|skip| skip == name)))
    })
}

fn has_hidden_file_name(path: &Path, skip_hidden: bool) -> bool {
    skip_hidden
        && path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|name| name.starts_with('.'))
}

#[cfg(test)]
#[path = "../tests/unit/discover.rs"]
mod tests;
