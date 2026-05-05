//! Discord memory runtime-status reply branch for text and JSON rendering.

mod json;
mod readiness;
mod text;

pub(super) use json::format_downstream_admission_status_json;
pub(super) use json::format_memory_runtime_status_json;
pub(super) use text::format_downstream_admission_status_lines;
pub(super) use text::format_memory_runtime_status_lines;
