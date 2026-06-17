//! `search::repo_search::ast` exposes retired local AST search adapters.

mod engine;
mod language;
mod provider;

#[cfg(test)]
pub(crate) use self::engine::{RepoAstAnalysisIndex, build_repo_ast_analysis_index_from_checkout};
pub(crate) use self::engine::{
    ast_pattern_requests_generic_analysis, has_generic_ast_language_filters,
    search_repo_ast_analysis_hits, search_repo_ast_pattern_hits,
};
pub use self::engine::{
    repository_generic_ast_lang_for_path, repository_supports_generic_ast_analysis,
};
