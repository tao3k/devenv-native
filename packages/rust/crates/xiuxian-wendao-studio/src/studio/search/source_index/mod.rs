//! Coordinates the Studio studio search source index branch and keeps its child modules behind one documented reasoning-tree boundary.

mod filters;
mod navigation;
mod symbols;

#[cfg(test)]
pub(crate) use symbols::build_source_symbol_hits;
pub(crate) use symbols::{build_symbol_index, build_symbol_index_from_source_symbol_hits};
