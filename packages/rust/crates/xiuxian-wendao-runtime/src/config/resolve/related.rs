//! Related-node configuration resolver for runtime graph behavior.

use crate::config::LinkGraphRelatedRuntimeConfig;
use crate::config::constants::{
    LINK_GRAPH_RELATED_MAX_CANDIDATES_ENV, LINK_GRAPH_RELATED_MAX_PARTITIONS_ENV,
    LINK_GRAPH_RELATED_TIME_BUDGET_MS_ENV,
};
use crate::settings::{
    first_non_empty, get_setting_string, parse_positive_f64, parse_positive_usize,
};
use serde_yaml::Value;

/// Resolve related-query runtime settings from merged Wendao configuration.
#[must_use]
pub fn resolve_link_graph_related_runtime_with_settings(
    settings: &Value,
) -> LinkGraphRelatedRuntimeConfig {
    let mut resolved = LinkGraphRelatedRuntimeConfig::default();

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.related.max_candidates"),
        std::env::var(LINK_GRAPH_RELATED_MAX_CANDIDATES_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
    {
        resolved.max_candidates = value;
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.related.max_partitions"),
        std::env::var(LINK_GRAPH_RELATED_MAX_PARTITIONS_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
    {
        resolved.max_partitions = value;
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.related.time_budget_ms"),
        std::env::var(LINK_GRAPH_RELATED_TIME_BUDGET_MS_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_f64)
    {
        resolved.time_budget_ms = value;
    }

    resolved
}

#[cfg(test)]
#[path = "../../../tests/unit/config/resolve/related.rs"]
mod tests;
