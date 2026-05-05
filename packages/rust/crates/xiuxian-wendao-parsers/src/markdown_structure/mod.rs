//! Internal Markdown structure extraction shared by parser owners.

mod api;
mod target_scan;
mod types;

pub(crate) use api::parse_markdown_document_metadata;
pub(crate) use api::parse_markdown_structure;
pub(crate) use types::{MarkdownStructuralItem, MarkdownStructure};
