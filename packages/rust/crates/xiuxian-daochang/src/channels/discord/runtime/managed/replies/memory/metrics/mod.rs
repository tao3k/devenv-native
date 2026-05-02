//! Discord memory metrics reply branch for text and JSON rendering.

mod json;
mod text;

pub(super) use json::format_memory_recall_metrics_json;
pub(super) use text::format_memory_recall_metrics_lines;
