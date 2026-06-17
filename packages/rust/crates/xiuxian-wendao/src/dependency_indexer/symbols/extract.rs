//! `dependency_indexer::symbols::extract` owns Wendao dependency indexer symbols extract behavior.

use std::path::Path;

use super::ExternalSymbol;

/// Extract dependency symbols from a source file (synchronous).
///
/// # Errors
///
/// Returns I/O errors when reading `path`.
pub fn extract_dependency_symbols(
    path: &Path,
    lang: &str,
) -> Result<Vec<ExternalSymbol>, std::io::Error> {
    extract_dependency_symbols_impl(path, lang)
}

#[cfg(feature = "search-runtime")]
fn extract_dependency_symbols_impl(
    _path: &Path,
    _lang: &str,
) -> Result<Vec<ExternalSymbol>, std::io::Error> {
    Ok(Vec::new())
}

#[cfg(not(feature = "search-runtime"))]
fn extract_dependency_symbols_impl(
    _path: &Path,
    _lang: &str,
) -> Result<Vec<ExternalSymbol>, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "dependency symbol extraction requires the search-runtime feature",
    ))
}
