//! Dependency Indexer - Index external Rust crate dependencies for API lookup.

mod config;
#[path = "indexer/mod.rs"]
mod indexer;
#[path = "symbols/mod.rs"]
mod symbols;

pub use config::{ConfigExternalDependency, DependencyConfig as DependencyBuildConfig};
pub use indexer::{DependencyConfig, DependencyIndexResult, DependencyIndexer, DependencyStats};
pub use symbols::{ExternalSymbol, SymbolIndex, SymbolKind, extract_symbols};
