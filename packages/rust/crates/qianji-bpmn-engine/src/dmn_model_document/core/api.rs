//! Public dmn model document core contracts for BPMN/DMN engine integration.

use super::DmnFunctionDefinitionSnapshot;

/// Snapshot of one top-level DMN `import`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnImportSnapshot {
    /// Optional import alias used by QName-style references.
    pub name: Option<String>,
    /// Optional imported model namespace.
    pub namespace: Option<String>,
    /// Optional import location URI.
    pub location_uri: Option<String>,
    /// Optional imported model type URI.
    pub import_type: Option<String>,
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
    /// Optional direct invocable `variable` metadata preserved for this bounded slice.
    pub variable: Option<DmnVariableSnapshot>,
    /// Optional direct `encapsulatedLogic` placeholder preserved for this bounded slice.
    pub encapsulated_logic: Option<DmnFunctionDefinitionSnapshot>,
    /// Optional direct body literal-expression metadata preserved for this bounded slice.
    pub body: Option<DmnBusinessKnowledgeModelLiteralSnapshot>,
}

/// Snapshot of one direct top-level business-knowledge-model body expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnBusinessKnowledgeModelLiteralSnapshot {
    /// Optional stable literal-expression identifier.
    pub expression_id: Option<String>,
    /// Optional DMN `typeRef` metadata on the literal expression.
    pub type_ref: Option<String>,
    /// Optional direct text payload.
    pub text: Option<String>,
}

/// Snapshot of one top-level DMN `decisionService`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceSnapshot {
    /// Optional stable DMN decision-service identifier.
    pub decision_service_id: Option<String>,
    /// Optional human-readable decision-service name.
    pub name: Option<String>,
    /// Direct `outputDecision` references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_decisions: Vec<DmnDecisionServiceReferenceSnapshot>,
    /// Direct `encapsulatedDecision` references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub encapsulated_decisions: Vec<DmnDecisionServiceReferenceSnapshot>,
    /// Direct `inputDecision` references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_decisions: Vec<DmnDecisionServiceReferenceSnapshot>,
    /// Direct `inputData` references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_data: Vec<DmnDecisionServiceReferenceSnapshot>,
}

/// Snapshot of one direct decision-service `tDMNElementReference` placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionServiceReferenceSnapshot {
    /// Optional direct `href` payload preserved from the reference element.
    pub href: Option<String>,
    /// Local name of the reference element, such as `outputDecision`.
    pub reference_kind: String,
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
