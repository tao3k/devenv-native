use super::{
    DmnAssociationSnapshot, DmnBusinessKnowledgeModelSnapshot, DmnDecisionServiceSnapshot,
    DmnDmndiSnapshot, DmnElementCollectionSnapshot, DmnGroupSnapshot, DmnImportSnapshot,
    DmnInputDataSnapshot, DmnItemDefinitionSnapshot, DmnKnowledgeSourceSnapshot,
    DmnOrganizationUnitSnapshot, DmnPerformanceIndicatorSnapshot, DmnTextAnnotationSnapshot,
};

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
    /// Bounded top-level `import` metadata preserved from the document.
    pub imports: Vec<DmnImportSnapshot>,
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
