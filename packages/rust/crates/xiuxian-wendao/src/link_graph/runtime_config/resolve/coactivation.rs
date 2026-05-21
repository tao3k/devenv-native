//! `link_graph::runtime_config::resolve::coactivation` owns Wendao runtime config resolve coactivation behavior.

use crate::link_graph::runtime_config::models::LinkGraphCoactivationRuntimeConfig;
use crate::link_graph::runtime_config::settings::merged_wendao_settings;
use xiuxian_wendao_runtime::config::resolve_link_graph_coactivation_runtime_with_settings;

/// Resolve coactivation runtime config with blueprint-aligned keys.
///
/// Config keys follow the `living_brain_v2` blueprint:
/// `link_graph.saliency.coactivation.*`
#[must_use]
pub fn resolve_link_graph_coactivation_runtime() -> LinkGraphCoactivationRuntimeConfig {
    let settings = merged_wendao_settings();
    resolve_link_graph_coactivation_runtime_with_settings(&settings)
}
