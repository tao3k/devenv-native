//! Transport selector helpers for Julia link-graph compatibility.

use xiuxian_wendao_core::{
    artifacts::PluginArtifactSelector,
    capabilities::PluginProviderSelector,
    ids::{ArtifactId, CapabilityId, PluginId},
};

/// Stable plugin id used by the Julia compatibility path.
pub const JULIA_PLUGIN_ID: &str = "xiuxian-julia-core";
/// Stable capability id used by the Julia rerank compatibility path.
pub const JULIA_RERANK_CAPABILITY_ID: &str = "rerank";
/// Stable capability id used by the Julia capability-manifest path.
pub const JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID: &str = "plugin-capabilities";
/// Stable capability id used by the Julia parser-summary compatibility path.
pub const JULIA_PARSER_SUMMARY_CAPABILITY_ID: &str = "parser-summary";
/// Stable capability id used by the Julia graph-structural compatibility path.
pub const JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID: &str = "graph-structural";
/// Stable artifact id used by the Julia deployment compatibility path.
pub const JULIA_DEPLOYMENT_ARTIFACT_ID: &str = "deployment";

/// Build the canonical rerank capability selector for the Julia plugin.
#[must_use]
pub fn julia_rerank_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(JULIA_RERANK_CAPABILITY_ID.to_string()),
        provider: PluginId(JULIA_PLUGIN_ID.to_string()),
    }
}

/// Build the canonical capability-manifest selector for the Julia plugin.
#[must_use]
pub fn julia_capability_manifest_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID.to_string()),
        provider: PluginId(JULIA_PLUGIN_ID.to_string()),
    }
}

/// Build the canonical parser-summary selector for the Julia plugin.
#[must_use]
pub fn julia_parser_summary_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(JULIA_PARSER_SUMMARY_CAPABILITY_ID.to_string()),
        provider: PluginId(JULIA_PLUGIN_ID.to_string()),
    }
}

/// Build the canonical graph-structural capability selector for the Julia
/// plugin.
#[must_use]
pub fn julia_graph_structural_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.to_string()),
        provider: PluginId(JULIA_PLUGIN_ID.to_string()),
    }
}

/// Build the canonical deployment-artifact selector for the Julia plugin.
#[must_use]
pub fn julia_deployment_artifact_selector() -> PluginArtifactSelector {
    PluginArtifactSelector {
        plugin_id: PluginId(JULIA_PLUGIN_ID.to_string()),
        artifact_id: ArtifactId(JULIA_DEPLOYMENT_ARTIFACT_ID.to_string()),
    }
}
