//! `zhenfa_router::native::audit::fuzzy_suggest::sources` owns Wendao audit fuzzy suggest sources behavior.

use std::path::Path;

use super::language::{CodeLanguageId, code_language_id_from_path};
use super::types::SourceFile;
use walkdir::WalkDir;

/// Resolve source files from directory paths.
///
/// This is a simple implementation that scans for common source file extensions.
/// For more sophisticated discovery, use the `dependency_indexer`.
#[must_use]
pub fn resolve_source_files(paths: &[&Path], language_id: &CodeLanguageId) -> Vec<SourceFile> {
    paths
        .iter()
        .flat_map(|path| resolve_source_files_for_path(path, language_id))
        .collect()
}

fn resolve_source_files_for_path(path: &Path, language_id: &CodeLanguageId) -> Vec<SourceFile> {
    if path.is_file() {
        return source_file_from_path(path, language_id)
            .into_iter()
            .collect();
    }
    if !path.is_dir() {
        return Vec::new();
    }

    WalkDir::new(path)
        .into_iter()
        .filter_entry(should_descend)
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| source_file_from_path(entry.path(), language_id))
        .collect()
}

fn source_file_from_path(path: &Path, language_id: &CodeLanguageId) -> Option<SourceFile> {
    code_language_id_from_path(path)
        .filter(|resolved| *resolved == language_id.as_str())
        .and_then(|_| std::fs::read_to_string(path).ok())
        .map(|content| SourceFile {
            path: path.to_string_lossy().into_owned(),
            content,
        })
}

fn should_descend(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return true;
    }

    let Some(name) = entry.file_name().to_str() else {
        return false;
    };
    !matches!(
        name,
        ".git" | ".jj" | ".svn" | ".hg" | "target" | "node_modules"
    )
}
