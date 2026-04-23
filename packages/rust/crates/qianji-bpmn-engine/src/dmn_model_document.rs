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
    /// Bounded top-level `itemDefinition` metadata preserved from the document.
    pub item_definitions: Vec<DmnItemDefinitionSnapshot>,
    /// Number of top-level `inputData` elements discovered in the document.
    pub input_data_count: usize,
    /// Bounded top-level `inputData` metadata preserved from the document.
    pub input_data: Vec<DmnInputDataSnapshot>,
    /// Number of top-level `knowledgeSource` elements discovered in the document.
    pub knowledge_source_count: usize,
    /// Bounded top-level `knowledgeSource` metadata preserved from the document.
    pub knowledge_sources: Vec<DmnKnowledgeSourceSnapshot>,
    /// Number of top-level `businessKnowledgeModel` elements discovered in the document.
    pub business_knowledge_model_count: usize,
    /// Bounded top-level `businessKnowledgeModel` metadata preserved from the document.
    pub business_knowledge_models: Vec<DmnBusinessKnowledgeModelSnapshot>,
    /// Number of top-level `decisionService` elements discovered in the document.
    pub decision_service_count: usize,
    /// Bounded top-level `decisionService` metadata preserved from the document.
    pub decision_services: Vec<DmnDecisionServiceSnapshot>,
    /// Number of top-level `organizationUnit` elements discovered in the document.
    pub organization_unit_count: usize,
    /// Bounded top-level `organizationUnit` metadata preserved from the document.
    pub organization_units: Vec<DmnOrganizationUnitSnapshot>,
    /// Number of top-level `performanceIndicator` elements discovered in the document.
    pub performance_indicator_count: usize,
    /// Bounded top-level `performanceIndicator` metadata preserved from the document.
    pub performance_indicators: Vec<DmnPerformanceIndicatorSnapshot>,
    /// Number of top-level `textAnnotation` elements discovered in the document.
    pub text_annotation_count: usize,
    /// Bounded top-level `textAnnotation` metadata preserved from the document.
    pub text_annotations: Vec<DmnTextAnnotationSnapshot>,
    /// Number of top-level `association` elements discovered in the document.
    pub association_count: usize,
    /// Bounded top-level `association` metadata preserved from the document.
    pub associations: Vec<DmnAssociationSnapshot>,
    /// Number of top-level `elementCollection` elements discovered in the document.
    pub element_collection_count: usize,
    /// Bounded top-level `elementCollection` metadata preserved from the document.
    pub element_collections: Vec<DmnElementCollectionSnapshot>,
    /// Number of top-level `group` elements discovered in the document.
    pub group_count: usize,
    /// Bounded top-level `group` metadata preserved from the document.
    pub groups: Vec<DmnGroupSnapshot>,
    /// Number of top-level `dmndi:DMNDI` elements discovered in the document.
    pub dmndi_count: usize,
    /// Bounded top-level `dmndi:DMNDI` metadata preserved from the document.
    pub dmndi_blocks: Vec<DmnDmndiSnapshot>,
}

/// Snapshot of one top-level DMN `itemDefinition`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnItemDefinitionSnapshot {
    /// Optional stable DMN item-definition identifier.
    pub item_definition_id: Option<String>,
    /// Optional human-readable item-definition name.
    pub name: Option<String>,
    /// Optional DMN `typeRef` metadata on the item definition.
    pub type_ref: Option<String>,
    /// Optional parsed `isCollection` metadata on the item definition.
    pub is_collection: Option<bool>,
    /// Direct nested `itemComponent` metadata preserved for this bounded slice.
    pub item_components: Vec<DmnItemComponentSnapshot>,
}

/// Snapshot of one direct nested DMN `itemComponent`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnItemComponentSnapshot {
    /// Optional stable DMN item-component identifier.
    pub item_component_id: Option<String>,
    /// Optional human-readable item-component name.
    pub name: Option<String>,
    /// Optional DMN `typeRef` metadata on the item component.
    pub type_ref: Option<String>,
}

/// Snapshot of one top-level DMN `inputData`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInputDataSnapshot {
    /// Optional stable DMN input-data identifier.
    pub input_data_id: Option<String>,
    /// Optional human-readable input-data name.
    pub name: Option<String>,
    /// Optional direct nested `variable` metadata preserved for this bounded slice.
    pub variable: Option<DmnVariableSnapshot>,
}

/// Snapshot of one bounded DMN `variable` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnVariableSnapshot {
    /// Optional stable DMN variable identifier.
    pub variable_id: Option<String>,
    /// Optional human-readable variable name.
    pub name: Option<String>,
    /// Optional DMN `typeRef` metadata on the variable.
    pub type_ref: Option<String>,
}

/// Snapshot of one top-level DMN `knowledgeSource`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnKnowledgeSourceSnapshot {
    /// Optional stable DMN knowledge-source identifier.
    pub knowledge_source_id: Option<String>,
    /// Optional human-readable knowledge-source name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `businessKnowledgeModel`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnBusinessKnowledgeModelSnapshot {
    /// Optional stable DMN business-knowledge-model identifier.
    pub business_knowledge_model_id: Option<String>,
    /// Optional human-readable business-knowledge-model name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `decisionService`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceSnapshot {
    /// Optional stable DMN decision-service identifier.
    pub decision_service_id: Option<String>,
    /// Optional human-readable decision-service name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `organizationUnit`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnOrganizationUnitSnapshot {
    /// Optional stable DMN organization-unit identifier.
    pub organization_unit_id: Option<String>,
    /// Optional human-readable organization-unit name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `performanceIndicator`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnPerformanceIndicatorSnapshot {
    /// Optional stable DMN performance-indicator identifier.
    pub performance_indicator_id: Option<String>,
    /// Optional human-readable performance-indicator name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `textAnnotation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnTextAnnotationSnapshot {
    /// Optional stable DMN text-annotation identifier.
    pub text_annotation_id: Option<String>,
    /// Optional direct nested DMN text payload preserved for this bounded slice.
    pub text: Option<String>,
}

/// Snapshot of one top-level DMN `association`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnAssociationSnapshot {
    /// Optional stable DMN association identifier.
    pub association_id: Option<String>,
    /// Optional DMN `associationDirection` metadata.
    pub association_direction: Option<String>,
    /// Optional direct nested `sourceRef` payload preserved for this bounded slice.
    pub source_ref: Option<String>,
    /// Optional direct nested `targetRef` payload preserved for this bounded slice.
    pub target_ref: Option<String>,
}

/// Snapshot of one top-level DMN `elementCollection`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnElementCollectionSnapshot {
    /// Optional stable DMN element-collection identifier.
    pub element_collection_id: Option<String>,
    /// Optional human-readable element-collection name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `group`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnGroupSnapshot {
    /// Optional stable DMN group identifier.
    pub group_id: Option<String>,
    /// Optional human-readable group name.
    pub name: Option<String>,
}

/// Snapshot of one top-level DMN `dmndi:DMNDI` block.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDmndiSnapshot {
    /// Optional stable DMNDI block identifier.
    pub dmndi_id: Option<String>,
    /// Direct nested `DMNDiagram` placeholder metadata preserved for this bounded slice.
    pub diagrams: Vec<DmnDiagramSnapshot>,
}

/// Snapshot of one direct nested DMNDI `DMNDiagram`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDiagramSnapshot {
    /// Optional stable DMN diagram identifier.
    pub diagram_id: Option<String>,
    /// Number of direct nested `DMNShape` elements discovered for the diagram.
    pub shape_count: usize,
    /// Number of direct nested `DMNEdge` elements discovered for the diagram.
    pub edge_count: usize,
    /// Direct nested `DMNShape` placeholder metadata preserved for this bounded slice.
    pub shapes: Vec<DmnShapeSnapshot>,
    /// Direct nested `DMNEdge` placeholder metadata preserved for this bounded slice.
    pub edges: Vec<DmnEdgeSnapshot>,
}

/// Snapshot of one direct nested DMNDI `DMNShape`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnShapeSnapshot {
    /// Optional stable DMN shape identifier.
    pub shape_id: Option<String>,
    /// Optional referenced DMN element identifier.
    pub dmn_element_ref: Option<String>,
    /// Optional direct `isListedInputData` marker preserved for this bounded slice.
    pub is_listed_input_data: Option<bool>,
    /// Optional direct `isCollapsed` marker preserved for this bounded slice.
    pub is_collapsed: Option<bool>,
    /// Optional direct nested `dc:Bounds` placeholder preserved for this bounded slice.
    pub bounds: Option<DmnBoundsSnapshot>,
    /// Optional direct nested `DMNDecisionServiceDividerLine` placeholder preserved for this bounded slice.
    pub decision_service_divider_line: Option<DmnDecisionServiceDividerLineSnapshot>,
    /// Optional direct nested `DMNLabel` placeholder preserved for this bounded slice.
    pub label: Option<DmnLabelSnapshot>,
}

/// Snapshot of one direct nested `dc:Bounds` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnBoundsSnapshot {
    /// Optional direct `x` payload preserved from `dc:Bounds`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `dc:Bounds`.
    pub y: Option<String>,
    /// Optional direct `width` payload preserved from `dc:Bounds`.
    pub width: Option<String>,
    /// Optional direct `height` payload preserved from `dc:Bounds`.
    pub height: Option<String>,
}

/// Snapshot of one direct nested DMNDI `DMNEdge`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnEdgeSnapshot {
    /// Optional stable DMN edge identifier.
    pub edge_id: Option<String>,
    /// Optional referenced DMN element identifier.
    pub dmn_element_ref: Option<String>,
    /// Direct nested `di:waypoint` placeholders preserved for this bounded slice.
    pub waypoints: Vec<DmnWaypointSnapshot>,
    /// Optional direct nested `DMNLabel` placeholder preserved for this bounded slice.
    pub label: Option<DmnLabelSnapshot>,
}

/// Snapshot of one direct nested `di:waypoint` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnWaypointSnapshot {
    /// Optional direct `x` payload preserved from `di:waypoint`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `di:waypoint`.
    pub y: Option<String>,
}

/// Snapshot of one direct nested `DMNDecisionServiceDividerLine` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceDividerLineSnapshot {
    /// Direct nested `di:waypoint` placeholders preserved for this bounded slice.
    pub waypoints: Vec<DmnWaypointSnapshot>,
}

/// Snapshot of one direct nested DMNDI `DMNLabel`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnLabelSnapshot {
    /// Optional stable DMN label identifier.
    pub label_id: Option<String>,
    /// Optional direct nested `dc:Bounds` placeholder preserved for this bounded slice.
    pub bounds: Option<DmnBoundsSnapshot>,
    /// Optional direct `DMNLabel/Text` payload preserved for this bounded slice.
    pub text: Option<String>,
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
