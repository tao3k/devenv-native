//! Search backend integration for Studio API.

#[path = "definition/mod.rs"]
mod definition;
#[path = "handlers/mod.rs"]
pub(crate) mod handlers;
#[path = "observation_hints.rs"]
mod observation_hints;
#[path = "project_scope.rs"]
mod project_scope;
#[path = "source_index/mod.rs"]
mod source_index;
#[path = "support.rs"]
mod support;

pub(crate) use definition::resolve_definition_candidates;
pub use definition::{DefinitionResolveOptions, resolve_best_definition};
pub use handlers::{build_symbol_index, search_index_status};
pub(crate) use project_scope::project_metadata_for_path;
pub(crate) use source_index::build_symbol_index_from_ast_hits;

pub use handlers::build_ast_index;
#[cfg(test)]
pub(crate) use support::strip_option;
