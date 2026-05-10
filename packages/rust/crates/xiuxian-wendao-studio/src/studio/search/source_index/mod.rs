//! Coordinates the Studio studio search source index branch and keeps its child modules behind one documented reasoning-tree boundary.

mod ast;
mod filters;
mod markdown;
mod navigation;
mod symbols;

pub(crate) use ast::build_ast_index;
pub(crate) use symbols::{build_symbol_index, build_symbol_index_from_ast_hits};
