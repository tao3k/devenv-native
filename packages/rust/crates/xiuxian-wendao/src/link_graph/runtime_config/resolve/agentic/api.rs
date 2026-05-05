use crate::link_graph::runtime_config::models::LinkGraphAgenticRuntimeConfig;
use crate::link_graph::runtime_config::settings::merged_wendao_settings;
use xiuxian_wendao_runtime::config::resolve_link_graph_agentic_runtime_with_settings;

/// Resolve agentic link-graph runtime settings from merged Wendao configuration.
#[must_use]
pub fn resolve_link_graph_agentic_runtime() -> LinkGraphAgenticRuntimeConfig {
    let settings = merged_wendao_settings();
    resolve_link_graph_agentic_runtime_with_settings(&settings)
}
