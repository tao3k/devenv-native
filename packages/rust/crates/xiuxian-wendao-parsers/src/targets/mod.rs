//! Markdown target occurrence extraction.

mod api;
mod scan;
mod types;

pub use api::extract_targets;
pub use types::{MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind, TargetOccurrenceCore};
