//! Dependency Indexer - Main implementation for indexing external dependencies.
//!
//! Uses `fd` command for fast file finding.

#[path = "core/mod.rs"]
mod core;
mod files;
mod types;

pub use crate::dependency_indexer::{DependencyBuildConfig, ExternalSymbol, SymbolIndex};
pub use core::DependencyIndexer;
pub use types::{DependencyConfig, DependencyIndexResult, DependencyStats};
