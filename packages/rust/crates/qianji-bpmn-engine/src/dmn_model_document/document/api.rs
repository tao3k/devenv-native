//! Public dmn model document document contracts for BPMN/DMN engine integration.

use super::{DmnDecisionSnapshot, DmnRootSnapshot};

/// Snapshot of one DMN document discovered before executable contract checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDocumentSnapshot {
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Root metadata discovered from the DMN document.
    pub root: DmnRootSnapshot,
    /// Decision headers discovered in source order.
    pub decisions: Vec<DmnDecisionSnapshot>,
}

impl DmnDocumentSnapshot {
    /// Returns one decision snapshot by id.
    #[must_use]
    pub fn decision(&self, decision_id: &str) -> Option<&DmnDecisionSnapshot> {
        self.decisions
            .iter()
            .find(|decision| decision.decision_id == decision_id)
    }
}
