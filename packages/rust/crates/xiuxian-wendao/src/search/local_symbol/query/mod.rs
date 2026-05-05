#[path = "autocomplete/mod.rs"]
mod autocomplete;
#[path = "lookup/mod.rs"]
mod lookup;
#[path = "shared/mod.rs"]
mod shared;
#[cfg(test)]
#[path = "../../../../tests/unit/search/local_symbol/query/mod.rs"]
mod tests;

pub(crate) use autocomplete::autocomplete_local_symbols;
pub(crate) use lookup::search_local_symbols;
pub use shared::LocalSymbolSearchError;
pub use shared::restore_local_symbol_hits;
