//! Scheduler preflight helper surface.
//!
//! Semantic resolution stays rooted here while helper parsing and URI handling
//! remain leaf-owned under `scheduler/preflight/`.

use serde_json::Value;

#[path = "scheduler/preflight/context_path.rs"]
mod context_path;
#[path = "scheduler/preflight/mounts.rs"]
mod mounts;
#[path = "scheduler/preflight/query.rs"]
mod query;
#[path = "scheduler/preflight/semantic.rs"]
mod semantic;
#[path = "scheduler/preflight/wendao_uri.rs"]
mod wendao_uri;

pub(crate) use mounts::{RuntimeWendaoMount, install_runtime_wendao_mounts};

#[must_use]
pub(crate) fn context_value_to_text(value: &Value) -> Option<String> {
    context_path::context_value_to_text(value)
}

#[must_use]
pub(crate) fn lookup_context_path<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    context_path::lookup_context_path(context, path)
}

pub(crate) fn resolve_wendao_placeholders_in_context(context: &Value) -> Result<Value, String> {
    semantic::resolve_wendao_placeholders_in_context(context)
}

pub(crate) fn resolve_semantic_content(raw: &str, context: &Value) -> Result<String, String> {
    semantic::resolve_semantic_content(raw, context)
}

pub(crate) fn resolve_semantic_reference(raw: &str, context: &Value) -> Result<String, String> {
    semantic::resolve_semantic_reference(raw, context)
}

pub(crate) fn resolve_wendao_uri_with_zhenfa(uri: &str) -> Result<String, String> {
    wendao_uri::resolve_wendao_uri_with_zhenfa(uri)
}
