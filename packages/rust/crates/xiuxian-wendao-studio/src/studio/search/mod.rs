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

pub use definition::DefinitionResolveOptions;
pub(crate) use definition::resolve_best_definition;
pub(crate) use definition::resolve_definition_candidates;
pub use handlers::{build_symbol_index, search_index_status};
pub(crate) use project_scope::project_metadata_for_path;
#[cfg(test)]
pub(crate) use source_index::build_source_symbol_hits;
pub(crate) use source_index::build_symbol_index_from_source_symbol_hits;
#[cfg(test)]
pub(crate) use support::strip_option;
