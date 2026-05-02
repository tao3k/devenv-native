//! Discord session-context reply branch for status, budget, and memory output.

mod json;
mod mode;
mod text;

pub(in super::super) use json::format_session_context_snapshot_json;
pub(in super::super) use text::format_session_context_snapshot;
