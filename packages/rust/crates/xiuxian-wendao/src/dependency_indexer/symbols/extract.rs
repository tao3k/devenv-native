//! `dependency_indexer::symbols::extract` owns Wendao dependency indexer symbols extract behavior.

use std::path::Path;

use super::{ExternalSymbol, SymbolKind};
use xiuxian_code_intelligence::{
    CodeLanguageId, SymbolKind as CodeSymbolKind, extract_code_dependency_symbols,
};

/// Extract symbols from a source file (synchronous).
///
/// # Errors
///
/// Returns I/O errors when reading `path`.
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
