//! Command runner interfaces for Markdown and semantic lint flows.

mod entry;
#[cfg(feature = "semantic-sql")]
mod semantic_render;

#[cfg(feature = "semantic-sql")]
pub(crate) use entry::SemanticLintReport;
pub(crate) use entry::run_markdown_lint;
#[cfg(feature = "semantic-sql")]
pub(crate) use entry::run_semantic_lint;
