use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Root configuration structure for `wendao.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlConfig {
    #[serde(default)]
    pub(crate) imports: Vec<String>,
    #[serde(default)]
    pub(crate) gateway: WendaoTomlGatewayConfig,
    #[serde(default)]
    pub(crate) document_extract: WendaoTomlDocumentExtractConfig,
    #[serde(default)]
    pub(crate) episteme: WendaoTomlEpistemeConfig,
    #[serde(default)]
    pub(crate) sources: WendaoTomlSourcesConfig,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub(crate) wendaograph: WendaoTomlWendaoGraphConfig,
}

/// Episteme repository registry configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlEpistemeConfig {
    #[serde(default)]
    pub(crate) registries: BTreeMap<String, WendaoTomlEpistemeRegistryConfig>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// One thin episteme registry entry from `wendao.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlEpistemeRegistryConfig {
    #[serde(default)]
    pub(crate) path: Option<String>,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(default)]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    pub(crate) subdir: Option<String>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// Gateway-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlGatewayConfig {
    #[serde(default)]
    pub(crate) bind: Option<String>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// Document extraction worker configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlDocumentExtractConfig {
    #[serde(default)]
    pub(crate) endpoint: Option<String>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// `WendaoGraph` service configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlWendaoGraphConfig {
    #[serde(default)]
    pub(crate) ontology_read_model_quality: WendaoTomlWendaoGraphOntologyReadModelQualityConfig,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// `WendaoGraph` ontology read-model quality Flight service configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlWendaoGraphOntologyReadModelQualityConfig {
    #[serde(default)]
    pub(crate) base_url: Option<String>,
    #[serde(default)]
    pub(crate) timeout_seconds: Option<u64>,
    #[serde(default)]
    pub(crate) max_in_flight_requests: Option<u64>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// Normalized `WendaoGraph` ontology read-model quality endpoint settings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(test, feature = "julia"))]
pub(crate) struct WendaoGraphOntologyReadModelQualityEndpointConfig {
    pub(crate) base_url: String,
    pub(crate) timeout_seconds: Option<u64>,
    pub(crate) max_in_flight_requests: Option<u64>,
}

/// Source discovery configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlSourcesConfig {
    #[serde(default)]
    pub(crate) include_dirs: Vec<String>,
    #[serde(default)]
    pub(crate) projects: BTreeMap<String, WendaoTomlProjectConfig>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

/// Per-project configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlProjectConfig {
    #[serde(default)]
    pub(crate) root: Option<String>,
    #[serde(default)]
    pub(crate) dirs: Vec<String>,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(rename = "ref", default)]
    pub(crate) git_ref: Option<String>,
    #[serde(default)]
    pub(crate) refresh: Option<String>,
    #[serde(default)]
    pub(crate) plugins: Vec<WendaoTomlPluginEntry>,
    #[serde(default, flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum WendaoTomlPluginEntry {
    Id(String),
    Config(WendaoTomlPluginInlineConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WendaoTomlPluginInlineConfig {
    pub(crate) id: String,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

impl WendaoTomlPluginEntry {
    pub(crate) fn normalized_id(&self) -> Option<String> {
        match self {
            Self::Id(id) => normalize_plugin_id(id),
            Self::Config(config) => normalize_plugin_id(config.id.as_str()),
        }
    }
}

fn normalize_plugin_id(raw: &str) -> Option<String> {
    let plugin = raw.trim();
    if plugin.is_empty() {
        None
    } else {
        Some(plugin.to_string())
    }
}
