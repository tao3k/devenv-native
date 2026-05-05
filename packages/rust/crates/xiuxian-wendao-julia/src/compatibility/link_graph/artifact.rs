//! Link-graph compatibility artifact resolution helpers.

use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginArtifactSelector},
    capabilities::PluginCapabilityBinding,
};
use xiuxian_wendao_runtime::{
    artifacts::render_plugin_artifact_toml_for_selector_with,
    transport::negotiate_flight_transport_client_from_bindings,
};

use super::runtime::LinkGraphJuliaRerankRuntimeConfig;
use super::selectors::julia_deployment_artifact_selector;

/// Resolve the Julia deployment artifact payload from the runtime config.
#[must_use]
pub fn resolve_julia_deployment_artifact_payload(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> PluginArtifactPayload {
    let binding = runtime.rerank_provider_binding();
    attach_plugin_artifact_transport_diagnostics(
        runtime.plugin_artifact_payload(),
        binding.as_ref(),
    )
}

/// Resolve one Julia plugin artifact through the Julia rerank runtime config.
#[must_use]
pub fn resolve_julia_plugin_artifact_payload_for_selector(
    selector: &PluginArtifactSelector,
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Option<PluginArtifactPayload> {
    (selector == &julia_deployment_artifact_selector())
        .then(|| resolve_julia_deployment_artifact_payload(runtime))
}

/// Render a resolved Julia plugin artifact as pretty TOML.
///
/// # Errors
///
/// Returns an error when the resolved artifact cannot be serialized into TOML.
pub fn render_julia_plugin_artifact_toml_for_selector(
    selector: &PluginArtifactSelector,
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Result<Option<String>, toml::ser::Error> {
    render_plugin_artifact_toml_for_selector_with(selector, |selector| {
        resolve_julia_plugin_artifact_payload_for_selector(selector, runtime)
    })
}

fn attach_plugin_artifact_transport_diagnostics(
    mut artifact: PluginArtifactPayload,
    binding: Option<&PluginCapabilityBinding>,
) -> PluginArtifactPayload {
    let Some(binding) = binding else {
        return artifact;
    };

    match negotiate_flight_transport_client_from_bindings(std::slice::from_ref(binding)) {
        Ok(Some(transport)) => {
            let selection = transport.selection();
            artifact.selected_transport = Some(selection.selected_transport);
            artifact.fallback_from = selection.fallback_from;
            artifact
                .fallback_reason
                .clone_from(&selection.fallback_reason);
        }
        Ok(None) => {
            artifact.fallback_from = Some(binding.transport);
            artifact.fallback_reason = Some(format!(
                "configured transport {:?} is unavailable because the binding has no base_url",
                binding.transport
            ));
        }
        Err(error) => {
            artifact.fallback_from = Some(binding.transport);
            artifact.fallback_reason = Some(error);
        }
    }

    artifact
}
