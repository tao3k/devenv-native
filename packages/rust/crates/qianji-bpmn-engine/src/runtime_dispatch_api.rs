//! Public runtime host-dispatch coordination state.

use crate::dmn_model_api::DmnDecisionRef;
use crate::ir_index_api::BpmnNodeIndex;

/// Host work categories owned by the bridge layer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingHostWorkKind {
    /// Service-task dispatch.
    Service,
    /// User-task dispatch.
    User,
    /// Manual-task dispatch.
    Manual,
    /// Business-rule task dispatch backed by the DMN contract.
    BusinessRule,
}

/// Recoverable pending host work reference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PendingHostWork {
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// Owning BPMN node index.
    pub node_index: BpmnNodeIndex,
    /// Host work category.
    pub kind: PendingHostWorkKind,
    /// Optional DMN decision binding for business-rule work.
    pub decision: Option<DmnDecisionRef>,
    /// Optional host-generated work identifier.
    pub work_id: Option<String>,
}
