use crate::dmn_model_api::{
    DmnBusinessKnowledgeModelDefinition, DmnDecisionDefinition, DmnDecisionRef,
    DmnDecisionServiceDefinition, DmnInputDataDefinition,
};
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
    /// Optional engine-owned DMN input-data registry for bounded local input binding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_input_data: Vec<DmnInputDataDefinition>,
    /// Optional engine-owned DMN business-knowledge-model registry for later
    /// local knowledge lookup.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_business_knowledge_models: Vec<DmnBusinessKnowledgeModelDefinition>,
    /// Optional engine-owned DMN decision-service registry for bounded local
    /// business-rule alias execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_decision_services: Vec<DmnDecisionServiceDefinition>,
}

impl BpmnPackage {
    /// Creates a package shell.
    #[must_use]
    pub fn new(package_id: impl AsRef<str>, processes: Vec<BpmnProcessSpec>) -> Self {
        Self {
            package_id: Arc::<str>::from(package_id.as_ref()),
            processes,
            dmn_decisions: Vec::new(),
            dmn_input_data: Vec::new(),
            dmn_business_knowledge_models: Vec::new(),
            dmn_decision_services: Vec::new(),
        }
    }

    /// Attaches engine-owned DMN decision definitions to the package.
    #[must_use]
    pub fn with_dmn_decisions(mut self, dmn_decisions: Vec<DmnDecisionDefinition>) -> Self {
        self.dmn_decisions = dmn_decisions;
        self
    }

    /// Attaches engine-owned DMN input-data definitions to the package.
    #[must_use]
    pub fn with_dmn_input_data(mut self, dmn_input_data: Vec<DmnInputDataDefinition>) -> Self {
        self.dmn_input_data = dmn_input_data;
        self
    }

    /// Attaches engine-owned DMN business-knowledge-model definitions to the package.
    #[must_use]
    pub fn with_dmn_business_knowledge_models(
        mut self,
        dmn_business_knowledge_models: Vec<DmnBusinessKnowledgeModelDefinition>,
    ) -> Self {
        self.dmn_business_knowledge_models = dmn_business_knowledge_models;
        self
    }

    /// Attaches engine-owned DMN decision-service definitions to the package.
    #[must_use]
    pub fn with_dmn_decision_services(
        mut self,
        dmn_decision_services: Vec<DmnDecisionServiceDefinition>,
    ) -> Self {
        self.dmn_decision_services = dmn_decision_services;
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

    /// Returns the registered DMN input-data definitions owned by the package.
    #[must_use]
    pub fn dmn_input_data(&self) -> &[DmnInputDataDefinition] {
        &self.dmn_input_data
    }

    /// Returns the registered DMN business-knowledge-model definitions owned
    /// by the package.
    #[must_use]
    pub fn dmn_business_knowledge_models(&self) -> &[DmnBusinessKnowledgeModelDefinition] {
        &self.dmn_business_knowledge_models
    }

    /// Returns the registered DMN decision-service definitions owned by the
    /// package.
    #[must_use]
    pub fn dmn_decision_services(&self) -> &[DmnDecisionServiceDefinition] {
        &self.dmn_decision_services
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

    /// Finds one deterministic DMN decision-service definition for a
    /// business-rule reference.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnDecisionServiceReference`] when
    /// more than one registered decision service matches the provided
    /// reference.
    pub fn find_dmn_decision_service(
        &self,
        decision_ref: &DmnDecisionRef,
    ) -> Result<Option<&DmnDecisionServiceDefinition>> {
        let mut matches = self
            .dmn_decision_services
            .iter()
            .filter(|decision_service| decision_service.matches_reference(decision_ref));
        let Some(first_match) = matches.next() else {
            return Ok(None);
        };
        let additional_matches = matches.count();
        if additional_matches > 0 {
            return Err(BpmnEngineError::AmbiguousDmnDecisionServiceReference {
                decision_service_id: decision_ref.decision_id.to_string(),
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

    /// Finds one deterministic DMN input-data definition for one same-source id.
    #[must_use]
    pub fn find_dmn_input_data(
        &self,
        source_id: &str,
        input_data_id: &str,
    ) -> Option<&DmnInputDataDefinition> {
        self.dmn_input_data.iter().find(|input_data| {
            input_data.source_id.as_ref() == source_id
                && input_data.input_data_id.as_deref() == Some(input_data_id)
        })
    }

    /// Finds one deterministic DMN business-knowledge-model definition for one
    /// same-source id.
    #[must_use]
    pub fn find_dmn_business_knowledge_model(
        &self,
        source_id: &str,
        business_knowledge_model_id: &str,
    ) -> Option<&DmnBusinessKnowledgeModelDefinition> {
        self.dmn_business_knowledge_models
            .iter()
            .find(|business_knowledge_model| {
                business_knowledge_model.source_id.as_ref() == source_id
                    && business_knowledge_model
                        .business_knowledge_model_id
                        .as_deref()
                        == Some(business_knowledge_model_id)
            })
    }
}
