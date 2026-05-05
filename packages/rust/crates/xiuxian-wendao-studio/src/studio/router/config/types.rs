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
    pub(crate) link_graph: WendaoTomlLinkGraphConfig,
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

/// Link graph configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlLinkGraphConfig {
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
