//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::dependency_indexer::{ExternalSymbol, SymbolKind};

pub(super) fn symbol_kind_from_str(kind: &str) -> SymbolKind {
    match kind {
        "struct" => SymbolKind::Struct,
        "enum" => SymbolKind::Enum,
        "trait" => SymbolKind::Trait,
        "fn" => SymbolKind::Function,
        "method" => SymbolKind::Method,
        "field" => SymbolKind::Field,
        "impl" => SymbolKind::Impl,
        "mod" => SymbolKind::Mod,
        "const" => SymbolKind::Const,
        "static" => SymbolKind::Static,
        "type" => SymbolKind::TypeAlias,
        _ => SymbolKind::Unknown,
    }
}

pub(super) fn symbol_kind_to_str(kind: &SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Trait => "trait",
        SymbolKind::Function => "fn",
        SymbolKind::Method => "method",
        SymbolKind::Field => "field",
        SymbolKind::Impl => "impl",
        SymbolKind::Mod => "mod",
        SymbolKind::Const => "const",
        SymbolKind::Static => "static",
        SymbolKind::TypeAlias => "type",
        SymbolKind::Unknown => "unknown",
    }
}

/// Convert `ExternalSymbol` to Python-compatible dict.
pub(super) fn symbol_to_dict(sym: &ExternalSymbol) -> serde_json::Value {
    serde_json::json!({
        "name": sym.name,
        "kind": format!("{:?}", sym.kind).to_lowercase(),
        "file": sym.file.to_string_lossy(),
        "line": sym.line,
        "crate_name": sym.crate_name,
    })
}
