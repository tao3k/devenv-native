//! Public ir package api contracts for BPMN/DMN engine integration.

use crate::bpmn_callable_api::{
    BpmnCallActivityBinding, BpmnCallableDefinition, BpmnCallableRegistry,
};
use crate::bpmn_collaboration_api::BpmnCollaborationHostEnvelope;
use crate::dmn_model_api::{
    DmnBusinessKnowledgeModelDefinition, DmnDecisionDefinition, DmnDecisionRef,
    DmnDecisionServiceDefinition, DmnImportDefinition, DmnImportSourceBinding,
    DmnInputDataDefinition, DmnSourceDefinition,
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
    /// Package-owned BPMN callable definition and binding registry.
    #[serde(default, skip_serializing_if = "BpmnCallableRegistry::is_empty")]
    pub callable_registry: BpmnCallableRegistry,
    /// Package-owned BPMN collaboration host envelope.
    #[serde(
        default,
        skip_serializing_if = "BpmnCollaborationHostEnvelope::is_empty"
    )]
    pub collaboration_host_envelope: BpmnCollaborationHostEnvelope,
    /// Optional package-owned non-executable DMN import registry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_imports: Vec<DmnImportDefinition>,
    /// Optional package-owned non-executable DMN source-root registry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dmn_source_definitions: Vec<DmnSourceDefinition>,
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
            callable_registry: BpmnCallableRegistry::default(),
            collaboration_host_envelope: BpmnCollaborationHostEnvelope::default(),
            dmn_imports: Vec::new(),
            dmn_source_definitions: Vec::new(),
            dmn_decisions: Vec::new(),
            dmn_input_data: Vec::new(),
            dmn_business_knowledge_models: Vec::new(),
            dmn_decision_services: Vec::new(),
        }
    }

    /// Attaches package-owned BPMN callable metadata.
    #[must_use]
    pub fn with_callable_registry(mut self, callable_registry: BpmnCallableRegistry) -> Self {
        self.callable_registry = callable_registry;
        self
    }

    /// Attaches package-owned BPMN collaboration host metadata.
    #[must_use]
    pub fn with_collaboration_host_envelope(
        mut self,
        collaboration_host_envelope: BpmnCollaborationHostEnvelope,
    ) -> Self {
        self.collaboration_host_envelope = collaboration_host_envelope;
        self
    }

    /// Attaches package-owned DMN source-root definitions.
    #[must_use]
    pub fn with_dmn_source_definitions(
        mut self,
        dmn_source_definitions: Vec<DmnSourceDefinition>,
    ) -> Self {
        self.dmn_source_definitions = dmn_source_definitions;
        self
    }

    /// Attaches package-owned DMN import definitions.
    #[must_use]
    pub fn with_dmn_imports(mut self, dmn_imports: Vec<DmnImportDefinition>) -> Self {
        self.dmn_imports = dmn_imports;
        self
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

    /// Returns the package-owned BPMN callable registry.
    #[must_use]
    pub fn callable_registry(&self) -> &BpmnCallableRegistry {
        &self.callable_registry
    }

    /// Returns the package-owned BPMN collaboration host envelope.
    #[must_use]
    pub fn collaboration_host_envelope(&self) -> &BpmnCollaborationHostEnvelope {
        &self.collaboration_host_envelope
    }

    /// Finds one callable definition by BPMN identifier.
    #[must_use]
    pub fn find_callable_definition(&self, callable_id: &str) -> Option<&BpmnCallableDefinition> {
        self.callable_registry.find_definition(callable_id)
    }

    /// Returns recorded process-target `callActivity` bindings.
    #[must_use]
    pub fn call_activity_bindings(&self) -> &[BpmnCallActivityBinding] {
        &self.callable_registry.call_activity_bindings
    }

    /// Returns the registered non-executable DMN import definitions.
    #[must_use]
    pub fn dmn_imports(&self) -> &[DmnImportDefinition] {
        &self.dmn_imports
    }

    /// Returns the registered non-executable DMN source-root definitions.
    #[must_use]
    pub fn dmn_source_definitions(&self) -> &[DmnSourceDefinition] {
        &self.dmn_source_definitions
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

    /// Returns package-owned DMN import declarations from one declaring source.
    #[must_use]
    pub fn dmn_imports_for_source(&self, source_id: &str) -> Vec<&DmnImportDefinition> {
        self.dmn_imports
            .iter()
            .filter(|dmn_import| dmn_import.is_declared_by(source_id))
            .collect()
    }

    /// Finds one deterministic DMN import by declaring source and import alias.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnImportReference`] when more than
    /// one import declared by the source uses the requested alias.
    pub fn find_dmn_import_by_name(
        &self,
        source_id: &str,
        name: &str,
    ) -> Result<Option<&DmnImportDefinition>> {
        self.find_dmn_import_by(source_id, "name", name, DmnImportDefinition::has_name)
    }

    /// Finds one deterministic DMN import by declaring source and imported namespace.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnImportReference`] when more than
    /// one import declared by the source targets the requested namespace.
    pub fn find_dmn_import_by_namespace(
        &self,
        source_id: &str,
        namespace: &str,
    ) -> Result<Option<&DmnImportDefinition>> {
        self.find_dmn_import_by(
            source_id,
            "namespace",
            namespace,
            DmnImportDefinition::has_namespace,
        )
    }

    /// Finds one deterministic DMN import by declaring source and location URI.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnImportReference`] when more than
    /// one import declared by the source uses the requested location URI.
    pub fn find_dmn_import_by_location_uri(
        &self,
        source_id: &str,
        location_uri: &str,
    ) -> Result<Option<&DmnImportDefinition>> {
        self.find_dmn_import_by(
            source_id,
            "locationURI",
            location_uri,
            DmnImportDefinition::has_location_uri,
        )
    }

    /// Finds one deterministic DMN source-root definition by source id.
    #[must_use]
    pub fn find_dmn_source_definition(&self, source_id: &str) -> Option<&DmnSourceDefinition> {
        self.dmn_source_definitions
            .iter()
            .find(|source_definition| source_definition.has_source_id(source_id))
    }

    /// Finds one deterministic DMN source-root definition by DMN namespace.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnSourceNamespace`] when more than
    /// one registered source root declares the requested namespace.
    pub fn find_dmn_source_definition_by_namespace(
        &self,
        namespace: &str,
    ) -> Result<Option<&DmnSourceDefinition>> {
        let mut matches = self
            .dmn_source_definitions
            .iter()
            .filter(|source_definition| source_definition.has_namespace(namespace));
        let Some(first_match) = matches.next() else {
            return Ok(None);
        };
        let additional_matches = matches.count();
        if additional_matches > 0 {
            return Err(BpmnEngineError::AmbiguousDmnSourceNamespace {
                namespace: namespace.to_string(),
                count: additional_matches + 1,
            });
        }
        Ok(Some(first_match))
    }

    /// Resolves one package-owned import declaration to a bundled DMN source
    /// root by imported namespace only.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnSourceNamespace`] when the
    /// import namespace matches more than one bundled source root.
    pub fn resolve_dmn_import_source(
        &self,
        dmn_import: &DmnImportDefinition,
    ) -> Result<Option<&DmnSourceDefinition>> {
        let Some(namespace) = dmn_import.namespace.as_deref() else {
            return Ok(None);
        };
        self.find_dmn_source_definition_by_namespace(namespace)
    }

    /// Returns one owned metadata-only binding report for every package-owned
    /// DMN import declaration.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnEngineError::AmbiguousDmnSourceNamespace`] when any import
    /// namespace matches more than one bundled source root.
    pub fn dmn_import_source_bindings(&self) -> Result<Vec<DmnImportSourceBinding>> {
        self.dmn_imports
            .iter()
            .map(|dmn_import| {
                let source_definition = self.resolve_dmn_import_source(dmn_import)?.cloned();
                Ok(DmnImportSourceBinding::new(
                    dmn_import.clone(),
                    source_definition,
                ))
            })
            .collect()
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

    fn find_dmn_import_by(
        &self,
        source_id: &str,
        selector_kind: &'static str,
        selector_value: &str,
        mut matches_selector: impl FnMut(&DmnImportDefinition, &str) -> bool,
    ) -> Result<Option<&DmnImportDefinition>> {
        let mut matches = self.dmn_imports.iter().filter(|dmn_import| {
            dmn_import.is_declared_by(source_id) && matches_selector(dmn_import, selector_value)
        });
        let Some(first_match) = matches.next() else {
            return Ok(None);
        };
        let additional_matches = matches.count();
        if additional_matches > 0 {
            return Err(BpmnEngineError::AmbiguousDmnImportReference {
                source_id: source_id.to_string(),
                selector_kind,
                selector_value: selector_value.to_string(),
                count: additional_matches + 1,
            });
        }
        Ok(Some(first_match))
    }
}
