//! Extract symbols from Rust/Python source files using omni-tags.

mod extract;
#[path = "index/mod.rs"]
mod index;
mod model;

pub use extract::extract_symbols;
pub use index::SymbolIndex;
pub use model::{ExternalSymbol, SymbolKind};

#[cfg(test)]
#[path = "../../../tests/unit/dependency_indexer/symbols/mod.rs"]
mod tests;
