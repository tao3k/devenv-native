use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Minimal normalized contract snapshot consumed by the runtime loader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(in crate::lint) struct MarkdownLintDiagnosticContractSnapshot {
    /// Stable contract identifier.
    pub id: String,
    /// Checked-in contract version.
    pub version: u32,
    /// Supported task types for this contract.
    pub task_types: Vec<String>,
    /// CLI invocation surface.
    pub cli: MarkdownLintCliContractSnapshot,
    /// Output schema metadata for rendered reports.
    pub output: MarkdownLintDiagnosticOutputSnapshot,
    /// Canonical parameter list.
    pub params: Vec<MarkdownLintContractParamSnapshot>,
    /// Ordered rendering rules covered by the snapshot.
    pub rules: Vec<MarkdownLintRuleContractSnapshot>,
}

/// CLI invocation surface for the markdown lint capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MarkdownLintCliContractSnapshot {
    /// Fixed command argv prefix.
    pub argv: Vec<String>,
    /// Ordered positional parameter names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positionals: Vec<String>,
    /// Canonical parameter to CLI flag mapping.
    pub flags: BTreeMap<String, String>,
}

/// Output metadata for the checked-in markdown lint contract snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MarkdownLintDiagnosticOutputSnapshot {
    /// Stable report format identifier.
    pub format: String,
    /// Sibling schema asset filename.
    pub schema: String,
}

/// Canonical parameter description for the invocation contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MarkdownLintContractParamSnapshot {
    /// Canonical parameter name.
    pub name: String,
    #[serde(rename = "type")]
    /// Minimal scalar or collection type hint used by the contract surface.
    pub value_type: String,
    #[serde(default)]
    /// Whether the parameter is mandatory for authored invocations.
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal default value.
    pub default: Option<MarkdownLintContractDefaultValue>,
}

/// Minimal literal default value surface kept in `contract.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum MarkdownLintContractDefaultValue {
    /// String literal default.
    String(String),
}

/// One normalized markdown lint rule contract entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(in crate::lint) struct MarkdownLintRuleContractSnapshot {
    /// Stable lint rule code.
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal problem text.
    pub problem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional dynamic problem strategy name.
    pub problem_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal detail text.
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional dynamic detail strategy name.
    pub detail_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal found text.
    pub found: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional dynamic found strategy name.
    pub found_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal expected text.
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional dynamic expected strategy name.
    pub expected_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal tip text.
    pub tip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional dynamic tip strategy name.
    pub tip_strategy: Option<String>,
}
