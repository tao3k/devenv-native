use super::scalar::{resolve_bool, resolve_usize};
use crate::config::LinkGraphAgenticRuntimeConfig;
use crate::config::constants::{
    LINK_GRAPH_AGENTIC_SEARCH_INCLUDE_PROVISIONAL_DEFAULT_ENV,
    LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT_ENV,
};
use serde_yaml::Value;

pub(super) fn apply_search_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    if let Some(value) = resolve_bool(
        settings,
        "link_graph.agentic.search.include_provisional_default",
        LINK_GRAPH_AGENTIC_SEARCH_INCLUDE_PROVISIONAL_DEFAULT_ENV,
    ) {
        resolved.search_include_provisional_default = value;
    }

    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.search.provisional_limit",
        LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT_ENV,
    ) {
        resolved.search_provisional_limit = value;
    }
}
