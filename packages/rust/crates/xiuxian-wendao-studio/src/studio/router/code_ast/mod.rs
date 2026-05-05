//! Code-AST response builders and repository/path resolution helpers.

mod atoms;
mod blocks;
mod generic;
mod resolve;
mod response;

pub(crate) use atoms::{RetrievalChunkLineExt, build_code_ast_retrieval_atom};
pub(crate) use blocks::build_code_block_retrieval_atoms;
pub(crate) use generic::build_generic_code_ast_analysis_response;
pub use resolve::resolve_code_ast_repository_and_path;
pub(crate) use resolve::{
    focus_symbol_for_blocks, path_has_extension, repo_relative_path_matches,
    retrieval_semantic_type,
};
pub use response::build_code_ast_analysis_response;
