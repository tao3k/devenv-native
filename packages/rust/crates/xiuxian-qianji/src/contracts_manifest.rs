//! Contracts manifest surface for `xiuxian-qianji`.

use std::{collections::BTreeMap, fmt, ops::Deref};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::consensus::ConsensusPolicy;

use super::{NodeLlmBinding, NodeQianhuanBinding};

/// Typed manifest node task-kind token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeTaskType(String);

impl NodeTaskType {
    /// Build a typed task-kind token from manifest input.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Return the manifest spelling for this task kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for NodeTaskType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for NodeTaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Definition of a node in the declarative manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct NodeDefinition {
    /// Unique identifier for the node.
    pub id: String,
    /// Type of task (e.g., knowledge, annotation).
    #[serde(alias = "kind")]
    pub task_type: NodeTaskType,
    /// Priority weight for scheduling.
    #[serde(default = "default_node_weight")]
    pub weight: f32,
    /// Task-specific parameters.
    #[serde(default = "default_node_params")]
    pub params: Value,
    /// Optional invocation contract reference for transport-native nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
    /// Optional HTTP method for `http_call` nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Optional HTTP path for `http_call` nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional HTTP base URL for relative `http_call` paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Optional HTTP query table for `http_call` nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<BTreeMap<String, Value>>,
    /// Optional argv vector for `cli_call` nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argv: Option<Vec<String>>,
    /// Optional node-level Qianhuan binding metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qianhuan: Option<NodeQianhuanBinding>,
    /// Optional node-level LLM tenant binding metadata.
    ///
    /// Backward compatibility:
    /// - preferred table: `[nodes.llm]`
    /// - legacy alias: `[nodes.llm_config]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "llm_config")]
    pub llm: Option<NodeLlmBinding>,
    /// Optional consensus policy for distributed voting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consensus: Option<ConsensusPolicy>,
}

/// Definition of an edge between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefinition {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Optional label for branch selection.
    pub label: Option<String>,
    /// Transition weight.
    pub weight: f32,
}

/// Declarative manifest for a Qianji workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QianjiManifest {
    /// Name of the pipeline.
    pub name: String,
    /// All node definitions.
    pub nodes: Vec<NodeDefinition>,
    /// All edge definitions.
    #[serde(default)]
    pub edges: Vec<EdgeDefinition>,
}

fn default_node_weight() -> f32 {
    1.0
}

fn default_node_params() -> Value {
    Value::Object(serde_json::Map::default())
}
