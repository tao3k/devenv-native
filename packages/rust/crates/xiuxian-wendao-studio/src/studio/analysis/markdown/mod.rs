//! Markdown analysis engine for Studio.

#[path = "compile/mod.rs"]
mod compile;
mod metadata;
mod text;

pub(crate) use self::compile::{CompiledDocument, compile_markdown_ir};
pub(crate) use self::metadata::build_markdown_document_metadata;
