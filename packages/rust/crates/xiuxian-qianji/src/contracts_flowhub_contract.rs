use serde::{Deserialize, Serialize};

/// Shared child-graph contract anchored by one Flowhub `qianji.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FlowhubStructureContract {
    /// Registered child graph nodes owned by the current manifest directory.
    #[serde(default)]
    pub register: Vec<String>,
    /// Required filesystem surfaces relative to the current manifest directory.
    ///
    /// Entries beginning with `*/` are expanded once per registered child.
    #[serde(default)]
    pub required: Vec<String>,
}
