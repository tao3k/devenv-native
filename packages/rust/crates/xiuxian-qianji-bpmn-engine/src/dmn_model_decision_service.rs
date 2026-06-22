//! Public dmn model decision service contracts for BPMN/DMN engine integration.

use crate::dmn_model_document::{DmnDecisionServiceReferenceSnapshot, DmnDecisionServiceSnapshot};
use crate::dmn_model_reference::DmnDecisionRef;
use std::sync::Arc;

/// One bounded executable DMN decision-service reference contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceReference {
    /// Direct reference element kind, such as `outputDecision`.
    pub reference_kind: Arc<str>,
    /// Direct href placeholder preserved from source.
    pub href: Option<Arc<str>>,
}

impl DmnDecisionServiceReference {
    /// Creates one bounded decision-service reference.
    #[must_use]
    pub fn new(reference_kind: impl AsRef<str>, href: Option<impl AsRef<str>>) -> Self {
        Self {
            reference_kind: (Arc::<str>::from(reference_kind.as_ref())),
            href: href.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Builds one bounded executable decision-service reference from one
    /// bounded snapshot entry.
    #[must_use]
    pub fn from_snapshot(snapshot: &DmnDecisionServiceReferenceSnapshot) -> Self {
        Self::new(snapshot.reference_kind.as_str(), snapshot.href.as_deref())
    }
}

/// One bounded executable DMN decision-service contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceDefinition {
    /// Source identifier used for diagnostics.
    pub source_id: Arc<str>,
    /// Stable top-level `decisionService` identifier when present.
    pub decision_service_id: Option<Arc<str>>,
    /// Human-readable top-level `decisionService` name when present.
    pub name: Option<Arc<str>>,
    /// Direct `outputDecision` references preserved in source order.
    pub output_decisions: Vec<DmnDecisionServiceReference>,
    /// Direct `encapsulatedDecision` references preserved in source order.
    pub encapsulated_decisions: Vec<DmnDecisionServiceReference>,
    /// Direct `inputDecision` references preserved in source order.
    pub input_decisions: Vec<DmnDecisionServiceReference>,
    /// Direct `inputData` references preserved in source order.
    pub input_data: Vec<DmnDecisionServiceReference>,
}

impl DmnDecisionServiceDefinition {
    /// Creates one bounded executable decision-service definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        decision_service_id: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            source_id: (Arc::<str>::from(source_id.as_ref())),
            decision_service_id: (decision_service_id
                .map(|value| Arc::<str>::from(value.as_ref()))),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            output_decisions: Vec::new(),
            encapsulated_decisions: Vec::new(),
            input_decisions: Vec::new(),
            input_data: Vec::new(),
        }
    }

    /// Attaches bounded direct `outputDecision` references.
    #[must_use]
    pub fn with_output_decisions(
        mut self,
        output_decisions: Vec<DmnDecisionServiceReference>,
    ) -> Self {
        self.output_decisions = output_decisions;
        self
    }

    /// Attaches bounded direct `encapsulatedDecision` references.
    #[must_use]
    pub fn with_encapsulated_decisions(
        mut self,
        encapsulated_decisions: Vec<DmnDecisionServiceReference>,
    ) -> Self {
        self.encapsulated_decisions = encapsulated_decisions;
        self
    }

    /// Attaches bounded direct `inputDecision` references.
    #[must_use]
    pub fn with_input_decisions(
        mut self,
        input_decisions: Vec<DmnDecisionServiceReference>,
    ) -> Self {
        self.input_decisions = input_decisions;
        self
    }

    /// Attaches bounded direct `inputData` references.
    #[must_use]
    pub fn with_input_data(mut self, input_data: Vec<DmnDecisionServiceReference>) -> Self {
        self.input_data = input_data;
        self
    }

    /// Builds one executable decision-service definition from one bounded
    /// snapshot entry.
    #[must_use]
    pub fn from_snapshot(
        source_id: impl AsRef<str>,
        snapshot: &DmnDecisionServiceSnapshot,
    ) -> Self {
        Self::new(
            source_id,
            snapshot.decision_service_id.as_deref(),
            snapshot.name.as_deref(),
        )
        .with_output_decisions(
            snapshot
                .output_decisions
                .iter()
                .map(DmnDecisionServiceReference::from_snapshot)
                .collect(),
        )
        .with_encapsulated_decisions(
            snapshot
                .encapsulated_decisions
                .iter()
                .map(DmnDecisionServiceReference::from_snapshot)
                .collect(),
        )
        .with_input_decisions(
            snapshot
                .input_decisions
                .iter()
                .map(DmnDecisionServiceReference::from_snapshot)
                .collect(),
        )
        .with_input_data(
            snapshot
                .input_data
                .iter()
                .map(DmnDecisionServiceReference::from_snapshot)
                .collect(),
        )
    }

    /// Returns whether the provided business-rule reference matches this
    /// registered decision service.
    #[must_use]
    pub fn matches_reference(&self, other: &DmnDecisionRef) -> bool {
        self.decision_service_id.as_deref() == Some(other.decision_id.as_ref())
            && other
                .source_id
                .as_deref()
                .is_none_or(|source_id| source_id == self.source_id.as_ref())
    }
}
