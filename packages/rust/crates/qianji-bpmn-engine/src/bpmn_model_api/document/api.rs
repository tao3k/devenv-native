//! Public bpmn model api document contracts for BPMN/DMN engine integration.

use super::collaboration::BpmnCollaborationSnapshot;
use super::process::BpmnProcessSnapshot;
use super::root::BpmnRootSnapshot;

/// Snapshot of one BPMN document discovered before executable subset checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDocumentSnapshot {
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Root metadata discovered from the BPMN document.
    pub root: BpmnRootSnapshot,
    /// Top-level collaboration metadata discovered in source order.
    pub collaborations: Vec<BpmnCollaborationSnapshot>,
    /// Top-level process metadata discovered in source order.
    pub processes: Vec<BpmnProcessSnapshot>,
}

impl BpmnDocumentSnapshot {
    /// Returns one process snapshot by id.
    #[must_use]
    pub fn process(&self, process_id: &str) -> Option<&BpmnProcessSnapshot> {
        self.processes
            .iter()
            .find(|process| process.process_id.as_deref() == Some(process_id))
    }
}
