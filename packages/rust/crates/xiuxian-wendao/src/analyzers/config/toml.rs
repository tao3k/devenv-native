use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlConfig {
    #[serde(default)]
    pub(crate) sources: WendaoTomlSourcesConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlSourcesConfig {
    #[serde(default)]
    pub(crate) projects: BTreeMap<String, WendaoTomlProjectConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct WendaoTomlProjectConfig {
    #[serde(default)]
    pub(crate) root: Option<String>,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(rename = "ref", default)]
    pub(crate) git_ref: Option<String>,
    #[serde(default)]
    pub(crate) refresh: Option<String>,
    #[serde(default)]
    pub(crate) plugins: Vec<WendaoTomlPluginEntry>,
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
    pub(crate) options: BTreeMap<String, serde_json::Value>,
}
