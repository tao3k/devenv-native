//! Search backend integration for Studio API.

mod definition;
pub mod handlers;
mod observation_hints;
mod project_scope;
mod source_index;
mod support;

pub use definition::{DefinitionResolveOptions, resolve_best_definition};
pub use handlers::{build_symbol_index, search_index_status};
pub(crate) use definition::resolve_definition_candidates;
pub(crate) use project_scope::{
    SearchProjectMetadata, configured_project_scopes, index_path_for_entry,
    project_metadata_for_path, resolve_project_root_path,
};
pub(crate) use source_index::{
    ast_search_lang, build_code_ast_hits_from_content, build_markdown_ast_hits_from_sections,
    is_markdown_path, markdown_scope_name, should_skip_entry,
};
pub(crate) use source_index::build_symbol_index_from_ast_hits;
pub(crate) use support::{infer_crate_name, score_reference_hit};

#[cfg(test)]
pub use handlers::build_ast_index;
#[cfg(test)]
pub(crate) use support::strip_option;
