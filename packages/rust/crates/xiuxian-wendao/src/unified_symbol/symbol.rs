//! `unified_symbol::symbol` owns Wendao unified symbol symbol behavior.

use serde::{Deserialize, Serialize};

/// Source type for a symbol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolSource {
    /// Symbol from the project itself.
    Project,
    /// Symbol from an external dependency.
    External(String), // crate name
}

/// A unified symbol that can represent both project and external symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct UnifiedSymbol {
    /// Symbol name.
    pub name: String,
    /// Symbol kind (for example `fn`, `struct`, `trait`).
    pub kind: String,
    /// Source location in `file:line` format.
    pub location: String, // file:line
    /// Source domain for this symbol.
    pub source: SymbolSource,
    /// Owning crate or package name.
    pub crate_name: String,
}

impl UnifiedSymbol {
    /// Create a project-local symbol record.
    #[must_use]
    pub fn new_project(name: &str, kind: &str, location: &str, crate_name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: kind.to_string(),
            location: location.to_string(),
            source: SymbolSource::Project,
            crate_name: crate_name.to_string(),
        }
    }

    /// Create an external dependency symbol record.
    #[must_use]
    pub fn new_external(name: &str, kind: &str, location: &str, crate_name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: kind.to_string(),
            location: location.to_string(),
            source: SymbolSource::External(crate_name.to_string()),
            crate_name: crate_name.to_string(),
        }
    }

    /// Returns true when the symbol comes from an external dependency.
    #[must_use]
    pub fn is_external(&self) -> bool {
        matches!(self.source, SymbolSource::External(_))
    }

    /// Returns true when the symbol comes from the project itself.
    #[must_use]
    pub fn is_project(&self) -> bool {
        matches!(self.source, SymbolSource::Project)
    }

    /// Returns crate name for both project and external symbols.
    #[must_use]
    pub fn crate_or_local(&self) -> &str {
        &self.crate_name
    }
}
