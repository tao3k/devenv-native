//! `link_graph::runtime_config::artifacts` owns Wendao link graph runtime config artifacts behavior.

#[cfg(all(
    feature = "julia",
    feature = "builtin-plugins",
    any(feature = "studio", feature = "zhenfa-router")
))]
use crate::link_graph::runtime_config::settings::merged_wendao_settings;
#[cfg(all(
    feature = "julia",
    feature = "builtin-plugins",
    any(feature = "studio", feature = "zhenfa-router")
))]
use xiuxian_wendao_builtin::resolve_builtin_plugin_artifact_for_selector_with_settings;
#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
use xiuxian_wendao_core::artifacts::{PluginArtifactPayload, PluginArtifactSelector};

/// Resolve one plugin artifact through the current link-graph runtime configuration.
#[must_use]
#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
pub fn resolve_link_graph_plugin_artifact_for_selector(
    selector: &PluginArtifactSelector,
) -> Option<PluginArtifactPayload> {
    #[cfg(all(feature = "julia", feature = "builtin-plugins"))]
    {
        let settings = merged_wendao_settings();
        resolve_builtin_plugin_artifact_for_selector_with_settings(selector, &settings)
    }

    #[cfg(not(all(feature = "julia", feature = "builtin-plugins")))]
    {
        let _ = selector;
        None
    }
}

/// Render a resolved link-graph plugin artifact as pretty TOML.
///
/// # Errors
///
/// Returns an error when the resolved artifact cannot be serialized into TOML.
#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
pub fn render_link_graph_plugin_artifact_toml_for_selector(
    selector: &PluginArtifactSelector,
) -> Result<Option<String>, toml::ser::Error> {
    resolve_link_graph_plugin_artifact_for_selector(selector)
        .map(|artifact| artifact.to_toml_string())
        .transpose()
}
