//! Markdown compilation pipeline for Studio.

mod compiler;
mod handlers;
mod retrieval;
mod syntax;
mod types;

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/analysis/markdown/compile/mod.rs"]
mod tests;

pub(crate) use self::compiler::compile_markdown_ir;
pub(crate) use self::types::CompiledDocument;
