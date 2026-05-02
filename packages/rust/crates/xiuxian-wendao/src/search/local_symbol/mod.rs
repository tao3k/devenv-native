#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(test)]
pub(crate) use build::ensure_local_symbol_index_started;
pub(crate) use build::ensure_local_symbol_index_started_with_scanned_files;
#[cfg(test)]
pub(crate) use build::plan_local_symbol_build;
#[cfg(test)]
pub(crate) use build::{LocalSymbolBuildError, publish_local_symbol_hits};
pub(crate) use query::{
    LocalSymbolSearchError, autocomplete_local_symbols, restore_local_symbol_hits,
    search_local_symbols,
};
