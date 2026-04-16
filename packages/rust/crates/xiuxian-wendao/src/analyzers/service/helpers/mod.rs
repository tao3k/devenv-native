//! Shared helper functions for repository intelligence service operations.

mod backlinks;
mod ecosystem;
mod example_relations;
mod path;
mod projection_lookup;
mod ranking;
mod scope;
mod uri;

#[cfg(test)]
#[path = "../../../../tests/unit/analyzers/service/helpers/mod.rs"]
mod tests;

pub(crate) use backlinks::{backlinks_for, documents_backlink_lookup};
pub(crate) use ecosystem::infer_ecosystem;
pub(crate) use example_relations::{
    example_relation_lookup, related_modules_for_example, related_symbols_for_example,
};
pub(crate) use path::hierarchy_segments_from_path;
pub(crate) use projection_lookup::{projection_page_lookup, projection_pages_for};
pub(crate) use ranking::{
    example_match_score, import_match_score, module_match_score, normalized_rank_score,
    symbol_match_score,
};
pub(crate) use scope::{
    docs_in_scope, documented_symbol_ids, resolve_module_scope, symbols_in_scope,
};
#[cfg(test)]
pub(crate) use uri::relation_kind_label;
pub(crate) use uri::{record_hierarchical_uri, repo_hierarchical_uri};
