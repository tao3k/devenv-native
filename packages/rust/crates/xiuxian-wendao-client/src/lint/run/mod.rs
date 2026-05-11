//! Command runner interfaces for Markdown and semantic lint flows.

mod entry;
mod semantic_render;

pub(crate) use entry::SemanticLintReport;
pub(crate) use entry::{run_markdown_lint, run_semantic_lint};
