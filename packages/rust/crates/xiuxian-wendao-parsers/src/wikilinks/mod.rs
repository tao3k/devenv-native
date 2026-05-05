//! Markdown wikilink extraction and literal parsing.

mod api;
mod types;

pub use api::{extract_wikilinks, parse_wikilink_literal};
pub use types::MarkdownWikiLink;
