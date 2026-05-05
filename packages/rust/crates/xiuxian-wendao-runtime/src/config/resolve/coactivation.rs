//! Coactivation configuration resolver for runtime host behavior.

use crate::config::LinkGraphCoactivationRuntimeConfig;
use crate::config::constants::{
    LINK_GRAPH_COACTIVATION_ALPHA_SCALE_ENV, LINK_GRAPH_COACTIVATION_ENABLED_ENV,
    LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE_ENV, LINK_GRAPH_COACTIVATION_MAX_HOPS_ENV,
    LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION_ENV,
    LINK_GRAPH_COACTIVATION_MAX_TOTAL_PROPAGATIONS_ENV,
    LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH_ENV,
};
use crate::settings::{
    first_non_empty, get_setting_string, parse_bool, parse_positive_f64, parse_positive_usize,
};
use serde_yaml::Value;

/// Resolve coactivation runtime config with blueprint-aligned keys.
#[must_use]
pub fn resolve_link_graph_coactivation_runtime_with_settings(
    settings: &Value,
) -> LinkGraphCoactivationRuntimeConfig {
    let mut resolved = LinkGraphCoactivationRuntimeConfig::default();
    let mut max_total_propagations_override: Option<usize> = None;

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.saliency.coactivation.enabled"),
        std::env::var(LINK_GRAPH_COACTIVATION_ENABLED_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_bool)
    {
        resolved.enabled = value;
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.saliency.coactivation.alpha_scale"),
        std::env::var(LINK_GRAPH_COACTIVATION_ALPHA_SCALE_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_f64)
    {
        resolved.alpha_scale = value;
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(
            settings,
            "link_graph.saliency.coactivation.max_neighbors_per_direction",
        ),
        std::env::var(LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
    {
        resolved.max_neighbors_per_direction = value;
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.saliency.coactivation.max_hops"),
        std::env::var(LINK_GRAPH_COACTIVATION_MAX_HOPS_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
    {
        resolved.max_hops = value.clamp(1, 2);
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(
            settings,
            "link_graph.saliency.coactivation.max_total_propagations",
        ),
        std::env::var(LINK_GRAPH_COACTIVATION_MAX_TOTAL_PROPAGATIONS_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
    {
        max_total_propagations_override = Some(value);
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(settings, "link_graph.saliency.coactivation.hop_decay_scale"),
        std::env::var(LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_f64)
    {
        resolved.hop_decay_scale = value.clamp(0.0, 1.0);
    }

    if let Some(value) = first_non_empty(&[
        get_setting_string(
            settings,
            "link_graph.saliency.coactivation.touch_queue_depth",
        ),
        std::env::var(LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH_ENV).ok(),
    ])
    .as_deref()
    .and_then(parse_positive_usize)
    {
        resolved.touch_queue_depth = value;
    }

    resolved.max_total_propagations = max_total_propagations_override.unwrap_or_else(|| {
        resolved
            .max_neighbors_per_direction
            .saturating_mul(2)
            .saturating_mul(resolved.max_hops)
    });

    resolved
}
