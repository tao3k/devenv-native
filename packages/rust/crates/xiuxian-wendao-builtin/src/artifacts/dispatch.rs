//! Dispatch helpers for resolving linked builtin plugin artifacts.

use serde_yaml::Value;
use xiuxian_wendao_core::artifacts::{PluginArtifactPayload, PluginArtifactSelector};

use xiuxian_wendao_julia::compatibility::link_graph::{
    LinkGraphJuliaRerankRuntimeConfig, julia_deployment_artifact_selector,
    render_julia_plugin_artifact_toml_for_selector,
    resolve_julia_plugin_artifact_payload_for_selector,
};

/// Resolve one builtin plugin artifact for the current linked builtin bundle.
#[must_use]
pub fn resolve_builtin_plugin_artifact_for_selector(
    selector: &PluginArtifactSelector,
    julia_rerank: &LinkGraphJuliaRerankRuntimeConfig,
) -> Option<PluginArtifactPayload> {
    if selector != &julia_deployment_artifact_selector() {
        return None;
    }

    resolve_julia_plugin_artifact_payload_for_selector(selector, julia_rerank)
}

/// Resolve one builtin plugin artifact for the current linked builtin bundle
/// from merged Wendao settings.
#[must_use]
pub fn resolve_builtin_plugin_artifact_for_selector_with_settings(
    selector: &PluginArtifactSelector,
    settings: &Value,
) -> Option<PluginArtifactPayload> {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::resolve_with_settings(settings);
    resolve_builtin_plugin_artifact_for_selector(selector, &runtime)
}

/// Render one builtin plugin artifact as pretty TOML for the current linked
/// builtin bundle.
///
/// # Errors
///
/// Returns an error when the selected builtin artifact cannot be serialized
/// into TOML.
pub fn render_builtin_plugin_artifact_toml_for_selector(
    selector: &PluginArtifactSelector,
    julia_rerank: &LinkGraphJuliaRerankRuntimeConfig,
) -> Result<Option<String>, toml::ser::Error> {
    if selector != &julia_deployment_artifact_selector() {
        return Ok(None);
    }

    render_julia_plugin_artifact_toml_for_selector(selector, julia_rerank)
}

/// Render one builtin plugin artifact as pretty TOML for the current linked
/// builtin bundle from merged Wendao settings.
///
/// # Errors
///
/// Returns an error when the selected builtin artifact cannot be serialized
/// into TOML.
pub fn render_builtin_plugin_artifact_toml_for_selector_with_settings(
    selector: &PluginArtifactSelector,
    settings: &Value,
) -> Result<Option<String>, toml::ser::Error> {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::resolve_with_settings(settings);
    render_builtin_plugin_artifact_toml_for_selector(selector, &runtime)
}
