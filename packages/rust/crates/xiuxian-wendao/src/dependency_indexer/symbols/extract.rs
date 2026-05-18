//! `dependency_indexer::symbols::extract` owns Wendao dependency indexer symbols extract behavior.

use std::path::Path;

use super::ExternalSymbol;
#[cfg(feature = "search-runtime")]
use super::SymbolKind;
#[cfg(feature = "search-runtime")]
use xiuxian_code_intelligence::{
    CodeLanguageId, SymbolKind as CodeSymbolKind, extract_code_dependency_symbols,
};

/// Extract symbols from a source file (synchronous).
///
/// # Errors
///
/// Returns I/O errors when reading `path`.
#[cfg(feature = "search-runtime")]
pub fn extract_symbols(path: &Path, lang: &str) -> Result<Vec<ExternalSymbol>, std::io::Error> {
    use std::fs::read_to_string;
    let content = read_to_string(path)?;
    Ok(
        extract_code_dependency_symbols(&content, &CodeLanguageId::from(lang))
            .into_iter()
            .map(|symbol| ExternalSymbol {
                name: symbol.name,
                kind: map_symbol_kind(&symbol.kind),
                file: path.to_path_buf(),
                line: symbol.line,
                crate_name: String::new(),
            })
            .collect(),
    )
}

/// Extract symbols from a source file (synchronous).
///
/// # Errors
///
/// Returns an unsupported-feature error when `search-runtime` is not enabled.
#[cfg(not(feature = "search-runtime"))]
pub fn extract_symbols(_path: &Path, _lang: &str) -> Result<Vec<ExternalSymbol>, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "dependency symbol extraction requires the search-runtime feature",
    ))
}

#[cfg(feature = "search-runtime")]
fn map_symbol_kind(kind: &CodeSymbolKind) -> SymbolKind {
    match kind {
        CodeSymbolKind::Struct | CodeSymbolKind::Class => SymbolKind::Struct,
        CodeSymbolKind::Enum => SymbolKind::Enum,
        CodeSymbolKind::Trait => SymbolKind::Trait,
        CodeSymbolKind::Function | CodeSymbolKind::AsyncFunction => SymbolKind::Function,
        CodeSymbolKind::Method => SymbolKind::Method,
        CodeSymbolKind::Impl => SymbolKind::Impl,
        CodeSymbolKind::Module => SymbolKind::Mod,
        CodeSymbolKind::Const => SymbolKind::Const,
        CodeSymbolKind::Static => SymbolKind::Static,
        CodeSymbolKind::TypeAlias | CodeSymbolKind::Interface => SymbolKind::TypeAlias,
        CodeSymbolKind::Unknown => SymbolKind::Unknown,
    }
}
