#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(any(test, feature = "test-support"))]
pub use build::LocalSymbolBuildError;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::ensure_local_symbol_index_started;
pub(crate) use build::ensure_local_symbol_index_started_with_scanned_files;
#[cfg(test)]
pub(crate) use build::plan_local_symbol_build;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::publish_local_symbol_hits;
pub use query::{LocalSymbolSearchError, restore_local_symbol_hits};
pub(crate) use query::{autocomplete_local_symbols, search_local_symbols};
