use super::scalar::{resolve_u64, resolve_usize};
use crate::config::LinkGraphAgenticRuntimeConfig;
use crate::config::constants::{
    LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES_ENV,
    LINK_GRAPH_AGENTIC_SUGGESTED_LINK_TTL_SECONDS_ENV,
};
use serde_yaml::Value;

pub(super) fn apply_suggested_link_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.suggested_link.max_entries",
        LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES_ENV,
    ) {
        resolved.suggested_link_max_entries = value;
    }

    resolved.suggested_link_ttl_seconds = resolve_u64(
        settings,
        "link_graph.agentic.suggested_link.ttl_seconds",
        LINK_GRAPH_AGENTIC_SUGGESTED_LINK_TTL_SECONDS_ENV,
    );
}
