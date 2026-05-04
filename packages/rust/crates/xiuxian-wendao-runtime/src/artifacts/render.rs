//! Artifact rendering helpers for runtime package outputs.

use xiuxian_wendao_core::artifacts::{PluginArtifactPayload, PluginArtifactSelector};

use super::{RuntimeArtifactIdRef, RuntimePluginIdRef};

/// Render a resolved plugin artifact as pretty TOML via a runtime-owned resolver.
///
/// # Errors
///
/// Returns an error when the resolved artifact cannot be serialized into TOML.
pub fn render_plugin_artifact_toml_with<F>(
    plugin_id: RuntimePluginIdRef<'_>,
    artifact_id: RuntimeArtifactIdRef<'_>,
    resolve: F,
) -> Result<Option<String>, toml::ser::Error>
where
    F: FnOnce(&str, &str) -> Option<PluginArtifactPayload>,
{
    resolve(plugin_id, artifact_id)
        .map(|artifact| toml::to_string_pretty(&artifact))
        .transpose()
}

/// Render a resolved plugin artifact as pretty TOML via a typed-selector resolver.
///
/// # Errors
///
/// Returns an error when the resolved artifact cannot be serialized into TOML.
pub fn render_plugin_artifact_toml_for_selector_with<F>(
    selector: &PluginArtifactSelector,
    resolve: F,
) -> Result<Option<String>, toml::ser::Error>
where
    F: FnOnce(&PluginArtifactSelector) -> Option<PluginArtifactPayload>,
{
    resolve(selector)
        .map(|artifact| toml::to_string_pretty(&artifact))
        .transpose()
}

#[cfg(test)]
#[path = "../../tests/unit/artifacts/render.rs"]
mod tests;
