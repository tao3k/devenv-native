//! Coordinates the Studio handlers analysis service branch and keeps its child modules behind one documented reasoning-tree boundary.

mod code_ast;
mod markdown;

pub(crate) use code_ast::load_code_ast_analysis_response;
pub(crate) use markdown::load_markdown_analysis_response;
