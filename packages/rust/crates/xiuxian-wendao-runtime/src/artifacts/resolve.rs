//! Artifact path resolution helpers for runtime package outputs.

use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginArtifactSelector},
    ids::{ArtifactId, PluginId},
};

use super::{RuntimeArtifactIdRef, RuntimePluginIdRef};

/// Resolve one plugin artifact through a runtime-owned typed-selector callback.
#[must_use]
pub fn resolve_plugin_artifact_with<F>(
    plugin_id: RuntimePluginIdRef<'_>,
    artifact_id: RuntimeArtifactIdRef<'_>,
    resolve: F,
) -> Option<PluginArtifactPayload>
where
    F: FnOnce(&PluginArtifactSelector) -> Option<PluginArtifactPayload>,
{
    let selector = PluginArtifactSelector {
        plugin_id: PluginId(plugin_id.to_string()),
        artifact_id: ArtifactId(artifact_id.to_string()),
    };
    resolve_plugin_artifact_for_selector_with(&selector, resolve)
}

/// Resolve one plugin artifact through a runtime-owned typed-selector callback.
#[must_use]
pub fn resolve_plugin_artifact_for_selector_with<F>(
    selector: &PluginArtifactSelector,
    resolve: F,
) -> Option<PluginArtifactPayload>
where
    F: FnOnce(&PluginArtifactSelector) -> Option<PluginArtifactPayload>,
{
    resolve(selector)
}

#[cfg(test)]
#[path = "../../tests/unit/artifacts/resolve.rs"]
mod tests;
