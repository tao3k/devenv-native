mod autocomplete;
mod lookup;
mod shared;
#[cfg(test)]
#[path = "../../../../tests/unit/search/local_symbol/query/mod.rs"]
mod tests;

pub(crate) use autocomplete::autocomplete_local_symbols;
pub(crate) use lookup::search_local_symbols;
pub(crate) use shared::LocalSymbolSearchError;
pub(crate) use shared::restore_local_symbol_hits;
