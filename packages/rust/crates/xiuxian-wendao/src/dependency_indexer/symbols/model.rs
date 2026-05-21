//! `dependency_indexer::symbols::model` owns Wendao dependency indexer symbols model behavior.

use std::path::PathBuf;

/// A single extracted symbol.
#[derive(Debug, Clone)]
pub struct ExternalSymbol {
    /// Symbol identifier.
    pub name: String,
    /// Symbol classification.
    pub kind: SymbolKind,
    /// File path containing this symbol.
    pub file: PathBuf,
    /// 1-based source line.
    pub line: usize,
    /// Source crate/package name.
    pub crate_name: String,
}

impl ExternalSymbol {
    /// Create a new `ExternalSymbol`.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn new(name: &str, kind: SymbolKind, file: &str, line: usize, crate_name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind,
            file: PathBuf::from(file),
            line,
            crate_name: crate_name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Supported symbol kinds extracted from source files.
pub enum SymbolKind {
    /// `struct` declaration.
    Struct,
    /// `enum` declaration.
    Enum,
    /// `trait` declaration.
    Trait,
    /// Free function declaration.
    Function,
    /// Method declaration.
    Method,
    /// Field declaration.
    Field,
    /// `impl` block.
    Impl,
    /// `mod` declaration.
    Mod,
    /// `const` declaration.
    Const,
    /// `static` declaration.
    Static,
    /// `type` alias.
    TypeAlias,
    /// Fallback for unknown syntax.
    Unknown,
}
