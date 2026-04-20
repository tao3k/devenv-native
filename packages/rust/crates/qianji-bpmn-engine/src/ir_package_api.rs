use crate::dmn_model_api::{DmnDecisionDefinition, DmnDecisionRef};
use crate::error::{BpmnEngineError, Result};
use crate::ir_process_lookup::usize_to_u32;
use crate::ir_process_spec::BpmnProcessSpec;
use std::sync::Arc;

/// Immutable BPMN package containing one or more process specs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnPackage {
    /// Package identifier.
    pub package_id: Arc<str>,
    /// Parsed processes in the package.
    pub processes: Vec<BpmnProcessSpec>,
    /// Optional engine-owned DMN decision registry for local business-rule execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_decisions: Vec<DmnDecisionDefinition>,
}

impl BpmnPackage {
    /// Creates a package shell.
    #[must_use]
    pub fn new(package_id: impl AsRef<str>, processes: Vec<BpmnProcessSpec>) -> Self {
        Self {
            package_id: Arc::<str>::from(package_id.as_ref()),
            processes,
            dmn_decisions: Vec::new(),
        }
    }

    /// Attaches engine-owned DMN decision definitions to the package.
    #[must_use]
    pub fn with_dmn_decisions(mut self, dmn_decisions: Vec<DmnDecisionDefinition>) -> Self {
        self.dmn_decisions = dmn_decisions;
        self
    }

    /// Finds a process position and spec by BPMN process identifier.
    #[must_use]
    pub fn find_process_position(&self, process_id: &str) -> Option<(u32, &BpmnProcessSpec)> {
        self.processes
            .iter()
            .enumerate()
            .find_map(|(index, process)| {
                (process.key.process_id.as_ref() == process_id)
                    .then_some((usize_to_u32(index, "process position"), process))
            })
    }

    /// Finds a process by BPMN process identifier.
    #[must_use]
    pub fn find_process(&self, process_id: &str) -> Option<&BpmnProcessSpec> {
        self.find_process_position(process_id)
            .map(|(_, process)| process)
    }

    /// Returns the registered DMN decision definitions owned by the package.
    #[must_use]
    pub fn dmn_decisions(&self) -> &[DmnDecisionDefinition] {
        &self.dmn_decisions
    }

    /// Finds one deterministic DMN decision definition for a business-rule reference.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnDecisionReference`] when more
    /// than one registered definition matches the provided reference.
    pub fn find_dmn_decision(
        &self,
        decision_ref: &DmnDecisionRef,
    ) -> Result<Option<&DmnDecisionDefinition>> {
        let mut matches = self
            .dmn_decisions
            .iter()
            .filter(|decision| decision.matches_reference(decision_ref));
        let Some(first_match) = matches.next() else {
            return Ok(None);
        };
        let additional_matches = matches.count();
        if additional_matches > 0 {
            return Err(BpmnEngineError::AmbiguousDmnDecisionReference {
                decision_id: decision_ref.decision_id.to_string(),
                source_id: decision_ref.source_id.as_ref().map(ToString::to_string),
                count: additional_matches + 1,
                source_suffix: decision_ref
                    .source_id
                    .as_ref()
                    .map(|source_id| format!(" in source '{source_id}'"))
                    .unwrap_or_default(),
            });
        }
        Ok(Some(first_match))
    }
}
