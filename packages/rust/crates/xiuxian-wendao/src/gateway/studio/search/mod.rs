//! Search backend integration for Studio API.

pub mod definition;
pub mod handlers;
pub mod observation_hints;
pub mod project_scope;
pub mod source_index;
pub mod support;

pub use handlers::{build_symbol_index, search_index_status};
pub(crate) use source_index::build_symbol_index_from_ast_hits;

#[cfg(test)]
pub use handlers::build_ast_index;
