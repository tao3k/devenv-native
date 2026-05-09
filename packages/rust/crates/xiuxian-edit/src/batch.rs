//! Batch Refactoring Engine - Heavy-Duty Parallel Processing
//!
//! The Ouroboros - Self-Eating Snake
//!
//! Provides parallel batch refactoring across entire codebases using rayon
//! and ignore for maximum performance. Python sends one command, Rust
//! processes thousands of files concurrently.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use xiuxian_ast::AstLanguage;

use crate::StructuralEditor;

/// Statistics for batch refactoring operations.
#[derive(Debug, Default)]
pub struct BatchRefactorStats {
    /// Number of files scanned
    pub files_scanned: usize,
    /// Number of files with changes
    pub files_changed: usize,
    /// Total number of replacements made
    pub replacements: usize,
    /// Errors encountered (path -> error message)
    pub errors: HashMap<String, String>,
    /// List of modified files
    pub modified_files: Vec<String>,
}

impl BatchRefactorStats {
    /// Create a new empty stats instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            files_changed: 0,
            replacements: 0,
            errors: HashMap::new(),
            modified_files: Vec::new(),
        }
    }
}

/// Configuration for batch refactoring.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// File glob pattern (e.g., "**/*.py")
    pub file_pattern: String,
    /// Whether to actually modify files (false) or just preview (true)
    pub dry_run: bool,
    /// Maximum file size in bytes (default 1MB)
    pub max_file_size: u64,
    /// Number of parallel workers (0 = auto-detect)
    pub workers: usize,
    /// Languages to process (empty = all detected)
    pub languages: Vec<String>,
    /// Skip directories matching these patterns
    pub skip_dirs: Vec<String>,
}

struct BatchRunState {
    files_scanned: AtomicUsize,
    files_changed: AtomicUsize,
    total_replacements: AtomicUsize,
    modified_files: Mutex<Vec<String>>,
    errors: Mutex<HashMap<String, String>>,
}

impl BatchRunState {
    fn new() -> Self {
        Self {
            files_scanned: AtomicUsize::new(0),
            files_changed: AtomicUsize::new(0),
            total_replacements: AtomicUsize::new(0),
            modified_files: Mutex::new(Vec::new()),
            errors: Mutex::new(HashMap::new()),
        }
    }

    fn into_stats(self) -> BatchRefactorStats {
        let mut stats = BatchRefactorStats::new();
        stats.files_scanned = self.files_scanned.load(Ordering::Relaxed);
        stats.files_changed = self.files_changed.load(Ordering::Relaxed);
        stats.replacements = self.total_replacements.load(Ordering::Relaxed);
        stats.modified_files = self
            .modified_files
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stats.modified_files.sort();
        stats.errors = self
            .errors
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stats
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            file_pattern: "**/*".to_string(),
            dry_run: true,
            max_file_size: 1_048_576,
            workers: 0,
            languages: Vec::new(),
            skip_dirs: vec![".git".to_string(), "node_modules".to_string()],
        }
    }
}

/// Detect programming language from file path.
fn detect_language(path: &Path) -> String {
    use xiuxian_ast::SupportLang;

    if let Some(lang) = SupportLang::from_path(path) {
        return format!("{lang:?}").to_lowercase();
    }
    "python".to_string()
}

impl StructuralEditor {
    /// Perform batch structural replace across a directory.
    ///
    /// This is the "heavy equipment" function that takes a directory and
    /// applies structural refactoring across all matching files in parallel.
    #[must_use]
    pub fn batch_replace(
        root: &Path,
        search_pattern: &str,
        rewrite_pattern: &str,
        config: &BatchConfig,
    ) -> BatchRefactorStats {
        batch_replace_internal(root, search_pattern, rewrite_pattern, config)
    }
}

fn batch_replace_internal(
    root: &Path,
    search_pattern: &str,
    rewrite_pattern: &str,
    config: &BatchConfig,
) -> BatchRefactorStats {
    let files = collect_batch_files(root, config);
    let state = BatchRunState::new();
    process_batch_files(files, search_pattern, rewrite_pattern, config, &state);
    state.into_stats()
}

fn collect_batch_files(root: &Path, config: &BatchConfig) -> Vec<PathBuf> {
    ignore::WalkBuilder::new(root)
        .threads(resolve_worker_count(config))
        .build()
        .filter_map(|entry| accepted_batch_file(entry.ok()?.path(), config))
        .collect()
}

fn resolve_worker_count(config: &BatchConfig) -> usize {
    if config.workers > 0 {
        config.workers
    } else {
        rayon::current_num_threads()
    }
}

fn accepted_batch_file(path: &Path, config: &BatchConfig) -> Option<PathBuf> {
    if path.is_file()
        && !is_skipped_path(path, &config.skip_dirs)
        && matches_glob(path, &config.file_pattern)
    {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn is_skipped_path(path: &Path, skip_dirs: &[String]) -> bool {
    path.parent().is_some_and(|parent| {
        parent.components().any(|component| {
            let std::path::Component::Normal(os_str) = component else {
                return false;
            };
            skip_dirs.iter().any(|skip_dir| os_str == skip_dir.as_str())
        })
    })
}

fn process_batch_files(
    files: Vec<PathBuf>,
    search_pattern: &str,
    rewrite_pattern: &str,
    config: &BatchConfig,
    state: &BatchRunState,
) {
    files.into_par_iter().for_each(|path| {
        process_batch_file(&path, search_pattern, rewrite_pattern, config, state);
    });
}

fn process_batch_file(
    path: &Path,
    search_pattern: &str,
    rewrite_pattern: &str,
    config: &BatchConfig,
    state: &BatchRunState,
) {
    state.files_scanned.fetch_add(1, Ordering::Relaxed);

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            record_error(state, path, format!("Read error: {error}"));
            return;
        }
    };

    match StructuralEditor::replace(
        &content,
        search_pattern,
        rewrite_pattern,
        &detect_language(path),
    ) {
        Ok(result) if result.count > 0 => {
            record_changed_file(state, path, result.count, config, &result.modified);
        }
        Ok(_) => {}
        Err(error) => record_error(state, path, format!("Edit error: {error}")),
    }
}

fn record_changed_file(
    state: &BatchRunState,
    path: &Path,
    replacement_count: usize,
    config: &BatchConfig,
    modified: &str,
) {
    state.files_changed.fetch_add(1, Ordering::Relaxed);
    state
        .total_replacements
        .fetch_add(replacement_count, Ordering::Relaxed);
    state
        .modified_files
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(path.display().to_string());

    if !config.dry_run
        && let Err(error) = std::fs::write(path, modified)
    {
        record_error(state, path, format!("Write error: {error}"));
    }
}

fn record_error(state: &BatchRunState, path: &Path, message: String) {
    state
        .errors
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(path.display().to_string(), message);
}

/// Check if a path matches a glob pattern (simplified implementation).
fn matches_glob(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    if pattern.starts_with("**/*") {
        let suffix = pattern.trim_start_matches("**/*");
        if suffix.is_empty() {
            return true;
        }
        path_str.ends_with(suffix) || path_str.contains(suffix)
    } else if let Some(stripped) = pattern.strip_prefix('*') {
        file_name.ends_with(stripped)
    } else if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 1 {
            return file_name == pattern;
        }
        let mut current = 0;
        for part in &parts {
            if let Some(pos) = file_name[current..].find(part) {
                current += pos + part.len();
            } else {
                return false;
            }
        }
        true
    } else {
        file_name == pattern
    }
}

#[cfg(test)]
#[path = "../tests/unit/batch.rs"]
mod tests;
