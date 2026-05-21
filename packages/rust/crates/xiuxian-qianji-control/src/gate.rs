//! Built-in deterministic evidence gates.

use std::collections::BTreeSet;

use crate::{EvidenceGate, GateName, GateResult, StepView};

/// Gate that requires every declared step evidence key to be covered.
#[derive(Debug, Clone)]
pub struct RequiredEvidenceGate {
    gate_name: GateName,
}

impl RequiredEvidenceGate {
    /// Creates the default required-evidence gate.
    ///
    /// # Errors
    ///
    /// Returns a control error if the gate name is blank.
    pub fn new(gate_name: impl Into<String>) -> crate::ControlResult<Self> {
        Ok(Self {
            gate_name: GateName::new(gate_name)?,
        })
    }
}

impl EvidenceGate for RequiredEvidenceGate {
    fn evaluate(&self, step: &StepView) -> GateResult {
        let covered = step
            .evidence
            .iter()
            .filter_map(|evidence| evidence.requirement_key.clone())
            .collect::<BTreeSet<_>>();
        let required = step
            .required_evidence
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let selected_required_evidence =
            required.intersection(&covered).cloned().collect::<Vec<_>>();
        let missing_required_evidence = required.difference(&covered).cloned().collect::<Vec<_>>();
        let required_evidence_covered = missing_required_evidence.is_empty();
        let reasons = if required_evidence_covered {
            Vec::new()
        } else {
            vec![format!(
                "missing required evidence: {}",
                missing_required_evidence.join(", ")
            )]
        };

        GateResult {
            gate_name: self.gate_name.clone(),
            passed: required_evidence_covered,
            required_evidence_covered,
            selected_required_evidence,
            missing_required_evidence,
            reasons,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}
