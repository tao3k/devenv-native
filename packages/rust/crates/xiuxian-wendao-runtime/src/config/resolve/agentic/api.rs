//! Public agentic configuration resolver API for Wendao runtime settings.

use super::apply;
use super::finalize;
use crate::config::LinkGraphAgenticRuntimeConfig;
use serde_yaml::Value;

/// Resolve agentic runtime settings from merged Wendao configuration.
#[must_use]
pub fn resolve_link_graph_agentic_runtime_with_settings(
    settings: &Value,
) -> LinkGraphAgenticRuntimeConfig {
    let mut resolved = LinkGraphAgenticRuntimeConfig::default();

    apply::apply_suggested_link_settings(settings, &mut resolved);
    apply::apply_search_settings(settings, &mut resolved);
    apply::apply_expansion_settings(settings, &mut resolved);
    apply::apply_execution_settings(settings, &mut resolved);

    finalize::finalize_execution_defaults(&mut resolved);
    resolved
}

#[cfg(test)]
#[path = "../../../../tests/unit/config/resolve/agentic/api.rs"]
mod tests;
