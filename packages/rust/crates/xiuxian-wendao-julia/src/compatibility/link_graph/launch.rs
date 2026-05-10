//! Launch configuration for Julia-backed link-graph compatibility routes.

use serde::{Deserialize, Serialize};
use xiuxian_wendao_core::artifacts::PluginLaunchSpec;

use crate::{JuliaContractMode, JuliaContractPath};

/// Additive Julia launch inputs resolved from Julia rerank runtime config.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LinkGraphJuliaSearchServiceDescriptor {
    /// Generic search service mode, usually `stream` or `table`.
    pub service_mode: Option<JuliaContractMode>,
    /// Optional path to Julia service TOML configuration.
    pub search_config_path: Option<JuliaContractPath>,
}

impl LinkGraphJuliaSearchServiceDescriptor {
    /// Build the generic plugin launch specification using the `WendaoSearch`
    /// service CLI arg mapping.
    #[must_use]
    pub fn plugin_launch_spec(&self, launcher_path: impl Into<String>) -> PluginLaunchSpec {
        let mut args = Vec::new();

        if let Some(service_mode) = self.service_mode.clone() {
            args.push("--mode".to_string());
            args.push(service_mode.into_string());
        }
        if let Some(config_path) = self.search_config_path.clone() {
            args.push("--config".to_string());
            args.push(config_path.into_string());
        }

        PluginLaunchSpec {
            launcher_path: launcher_path.into(),
            args,
        }
    }
}

/// Resolved Julia service launch manifest derived from runtime configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkGraphJuliaSearchLaunchManifest {
    /// Launcher path relative to the repository root.
    pub launcher_path: String,
    /// Ordered search-service CLI args.
    pub args: Vec<String>,
}

impl From<PluginLaunchSpec> for LinkGraphJuliaSearchLaunchManifest {
    fn from(value: PluginLaunchSpec) -> Self {
        Self {
            launcher_path: value.launcher_path,
            args: value.args,
        }
    }
}

impl From<LinkGraphJuliaSearchLaunchManifest> for PluginLaunchSpec {
    fn from(value: LinkGraphJuliaSearchLaunchManifest) -> Self {
        Self {
            launcher_path: value.launcher_path,
            args: value.args,
        }
    }
}
