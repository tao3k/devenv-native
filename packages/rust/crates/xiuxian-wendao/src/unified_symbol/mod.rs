//! Unified Symbol Index - Combines project symbols with external dependency symbols.
//!
//! This enables:
//! - Unified search across project + external deps
//! - Find where external APIs are used in your project
//! - Trace symbol origins and relationships
//!
//! # Usage
//!
//! ```rust
//! use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;
//!
//! let mut index = UnifiedSymbolIndex::new();
//!
//! // Add project symbols
//! index.add_project_symbol("my_func", "fn", "src/lib.rs:42", "mycrate");
//!
//! // Add external dependency symbols
//! index.add_external_symbol("spawn", "fn", "task_join_set.rs:1", "tokio");
//!
//! // Record usage of external symbol in project
//! index.record_external_usage("tokio", "spawn", "src/main.rs:10");
//!
//! // Search across both
//! let results = index.search_unified("spawn", 10);
//!
//! // Find where tokio::spawn is used in project
//! let usage = index.find_external_usage("tokio");
//! ```

#[path = "index/mod.rs"]
mod index;
mod stats;
mod symbol;

pub use index::UnifiedSymbolIndex;
pub use stats::UnifiedIndexStats;
pub use symbol::{SymbolSource, UnifiedSymbol};

#[cfg(test)]
#[path = "../../tests/unit/unified_symbol/mod.rs"]
mod tests;
