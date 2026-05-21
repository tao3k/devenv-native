//! `zhenfa_router::native::audit::fuzzy_suggest::sources` owns Wendao audit fuzzy suggest sources behavior.

use std::path::Path;

use super::types::SourceFile;
use xiuxian_code_intelligence::{CodeLanguageId, resolve_code_source_files_for_language_id};

/// Resolve source files from directory paths.
///
/// This is a simple implementation that scans for common source file extensions.
/// For more sophisticated discovery, use the `dependency_indexer`.
#[must_use]
pub fn resolve_source_files(paths: &[&Path], language_id: &CodeLanguageId) -> Vec<SourceFile> {
    resolve_code_source_files_for_language_id(paths, language_id)
        .into_iter()
        .map(|source| SourceFile {
            path: source.path,
            content: source.content,
        })
        .collect()
}
