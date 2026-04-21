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

/// Snapshot of the DMN document root metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRootSnapshot {
    /// Local name of the discovered root element.
    pub element_name: String,
    /// Optional `id` on the root element.
    pub definitions_id: Option<String>,
    /// Optional `name` on the root element.
    pub name: Option<String>,
    /// Optional DMN business namespace on the root element.
    pub namespace: Option<String>,
    /// Optional DMN model namespace URI discovered from `xmlns` attributes.
    pub model_namespace_uri: Option<String>,
    /// Optional DMN model-version hint derived from the namespace URI.
    pub model_version_hint: Option<String>,
    /// Number of top-level `import` elements discovered in the document.
    pub import_count: usize,
    /// Number of top-level `itemDefinition` elements discovered in the document.
    pub item_definition_count: usize,
    /// Number of top-level `inputData` elements discovered in the document.
    pub input_data_count: usize,
    /// Number of top-level `knowledgeSource` elements discovered in the document.
    pub knowledge_source_count: usize,
    /// Number of top-level `businessKnowledgeModel` elements discovered in the document.
    pub business_knowledge_model_count: usize,
    /// Number of top-level `decisionService` elements discovered in the document.
    pub decision_service_count: usize,
    /// Number of top-level `organizationUnit` elements discovered in the document.
    pub organization_unit_count: usize,
    /// Number of top-level `performanceIndicator` elements discovered in the document.
    pub performance_indicator_count: usize,
    /// Number of top-level `textAnnotation` elements discovered in the document.
    pub text_annotation_count: usize,
    /// Number of top-level `association` elements discovered in the document.
    pub association_count: usize,
    /// Number of top-level `elementCollection` elements discovered in the document.
    pub element_collection_count: usize,
    /// Number of top-level `group` elements discovered in the document.
    pub group_count: usize,
    /// Number of top-level `dmndi:DMNDI` elements discovered in the document.
    pub dmndi_count: usize,
}

/// Snapshot of one DMN decision header.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionSnapshot {
    /// Stable DMN decision identifier.
    pub decision_id: String,
    /// Optional human-readable decision name.
    pub name: Option<String>,
    /// Number of direct `allowedAnswers` children discovered for the decision.
    pub allowed_answers_count: usize,
    /// Number of direct `decisionMaker` children discovered for the decision.
    pub decision_maker_count: usize,
    /// Number of direct `decisionOwner` children discovered for the decision.
    pub decision_owner_count: usize,
    /// Number of nested `decisionTable` elements discovered for the decision.
    pub decision_table_count: usize,
    /// Number of direct `informationRequirement` children discovered for the decision.
    pub information_requirement_count: usize,
    /// Number of nested `requiredInput` children discovered under information requirements.
    pub required_input_count: usize,
    /// Number of nested `requiredDecision` children discovered under information requirements.
    pub required_decision_count: usize,
    /// Number of direct `knowledgeRequirement` children discovered for the decision.
    pub knowledge_requirement_count: usize,
    /// Number of nested `requiredKnowledge` children discovered under knowledge requirements.
    pub required_knowledge_count: usize,
    /// Number of direct `authorityRequirement` children discovered for the decision.
    pub authority_requirement_count: usize,
    /// Number of nested `requiredAuthority` children discovered under authority requirements.
    pub required_authority_count: usize,
    /// Number of direct `literalExpression` children discovered for the decision.
    pub literal_expression_count: usize,
    /// Number of direct `context` children discovered for the decision.
    pub context_count: usize,
    /// Number of direct `invocation` children discovered for the decision.
    pub invocation_count: usize,
    /// Number of direct `relation` children discovered for the decision.
    pub relation_count: usize,
    /// Number of direct `functionDefinition` children discovered for the decision.
    pub function_definition_count: usize,
    /// Number of direct `list` children discovered for the decision.
    pub list_count: usize,
}
