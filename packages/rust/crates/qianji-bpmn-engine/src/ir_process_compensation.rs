use crate::ir_index_api::BpmnNodeIndex;

/// One bounded compensation handler binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCompensationHandlerSpec {
    /// Boundary compensation event node index.
    pub boundary: BpmnNodeIndex,
    /// Original activity node index that may be compensated.
    pub activity: BpmnNodeIndex,
    /// Compensation activity node index.
    pub handler: BpmnNodeIndex,
}
