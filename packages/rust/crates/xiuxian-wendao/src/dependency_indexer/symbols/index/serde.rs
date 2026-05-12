//! Symbol index text serialization helpers.

use super::SymbolIndex;
use super::types::CrateSymbols;
use crate::dependency_indexer::symbols::{ExternalSymbol, SymbolKind};
use std::io::Write;

impl SymbolIndex {
    /// Serialize to JSON string.
    #[must_use]
    pub fn serialize(&self) -> String {
        serialize_index(self).unwrap_or_default()
    }

    /// Deserialize from JSON string.
    #[must_use]
    pub fn deserialize(&mut self, data: &str) -> bool {
        self.clear();

        for line in data.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 4 {
                continue;
            }

            let crate_name = parts[0];
            let name = parts[1];
            let kind_str = parts[2];
            let loc = parts[3];

            let kind = match kind_str {
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
            };

            // Parse file:line
            let mut file_parts = loc.rsplitn(2, ':');
            let file = file_parts.nth(1).unwrap_or(loc);
            let line = file_parts
                .nth(0)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1);

            let symbol = ExternalSymbol {
                name: name.to_string(),
                kind,
                file: std::path::PathBuf::from(file),
                line,
                crate_name: crate_name.to_string(),
            };

            self.add_symbols(crate_name, &[symbol]);
        }

        true
    }
}

fn serialize_index(index: &SymbolIndex) -> Option<String> {
    let mut output = Vec::new();
    for crate_symbols in &index.by_crate {
        write_crate_symbols(&mut output, crate_symbols).ok()?;
    }
    String::from_utf8(output).ok()
}

fn write_crate_symbols(output: &mut Vec<u8>, crate_symbols: &CrateSymbols) -> std::io::Result<()> {
    for symbol in &crate_symbols.symbols {
        write_symbol_line(output, &crate_symbols.name, symbol)?;
    }
    Ok(())
}

fn write_symbol_line(
    output: &mut Vec<u8>,
    crate_name: &str,
    symbol: &ExternalSymbol,
) -> std::io::Result<()> {
    let line = symbol.line;
    let file = symbol.file.to_string_lossy();
    writeln!(
        output,
        "{}|{}|{}|{}:{}",
        crate_name,
        symbol.name,
        symbol_kind_label(&symbol.kind),
        file,
        line
    )
}

fn symbol_kind_label(kind: &SymbolKind) -> &'static str {
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
