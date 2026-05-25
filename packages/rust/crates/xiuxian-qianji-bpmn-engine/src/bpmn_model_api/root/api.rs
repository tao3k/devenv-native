//! Public bpmn model api root contracts for BPMN/DMN engine integration.

use super::collaboration::{BpmnPartnerEntitySnapshot, BpmnPartnerRoleSnapshot};
use super::data::BpmnDataStoreSnapshot;
use super::definitions::{
    BpmnCategorySnapshot, BpmnCorrelationPropertySnapshot, BpmnEndPointSnapshot, BpmnErrorSnapshot,
    BpmnEscalationSnapshot, BpmnExtensionSnapshot, BpmnGlobalTaskSnapshot, BpmnImportSnapshot,
    BpmnInterfaceSnapshot, BpmnItemDefinitionSnapshot, BpmnMessageSnapshot,
    BpmnRelationshipSnapshot, BpmnResourceSnapshot, BpmnSignalSnapshot,
};
use super::di::BpmnDiagramSnapshot;
use crate::bpmn_parse_api::BpmnSourceFile;

/// Snapshot of BPMN `definitions` metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnRootSnapshot {
    /// Local name of the discovered root element.
    pub element_name: String,
    /// Optional `id` on the root element.
    pub definitions_id: Option<String>,
    /// Optional `name` on the root element.
    pub name: Option<String>,
    /// Optional BPMN `targetNamespace` metadata.
    pub target_namespace: Option<String>,
    /// Optional BPMN model namespace URI discovered from `xmlns` attributes.
    pub model_namespace_uri: Option<String>,
    /// Number of top-level `import` elements discovered in the document.
    #[serde(default)]
    pub import_count: usize,
    /// Bounded top-level `import` metadata preserved from the document.
    #[serde(default)]
    pub imports: Vec<BpmnImportSnapshot>,
    /// Number of top-level `extension` elements discovered in the document.
    #[serde(default)]
    pub extension_count: usize,
    /// Bounded top-level `extension` metadata preserved from the document.
    #[serde(default)]
    pub extensions: Vec<BpmnExtensionSnapshot>,
    /// Number of top-level `relationship` elements discovered in the document.
    #[serde(default)]
    pub relationship_count: usize,
    /// Bounded top-level `relationship` metadata preserved from the document.
    #[serde(default)]
    pub relationships: Vec<BpmnRelationshipSnapshot>,
    /// Number of top-level BPMN DI `BPMNDiagram` elements discovered in the document.
    #[serde(default)]
    pub diagram_count: usize,
    /// Bounded top-level BPMN DI diagram metadata preserved from the document.
    #[serde(default)]
    pub diagrams: Vec<BpmnDiagramSnapshot>,
    /// Number of top-level `collaboration` elements discovered in the document.
    pub collaboration_count: usize,
    /// Number of top-level `process` elements discovered in the document.
    pub process_count: usize,
    /// Number of top-level `itemDefinition` elements discovered in the document.
    #[serde(default)]
    pub item_definition_count: usize,
    /// Bounded top-level `itemDefinition` metadata preserved from the document.
    #[serde(default)]
    pub item_definitions: Vec<BpmnItemDefinitionSnapshot>,
    /// Number of top-level `message` elements discovered in the document.
    #[serde(default)]
    pub message_count: usize,
    /// Bounded top-level `message` metadata preserved from the document.
    #[serde(default)]
    pub messages: Vec<BpmnMessageSnapshot>,
    /// Number of top-level `interface` elements discovered in the document.
    #[serde(default)]
    pub interface_count: usize,
    /// Bounded top-level `interface` metadata preserved from the document.
    #[serde(default)]
    pub interfaces: Vec<BpmnInterfaceSnapshot>,
    /// Number of top-level `endPoint` elements discovered in the document.
    #[serde(default)]
    pub end_point_count: usize,
    /// Bounded top-level `endPoint` metadata preserved from the document.
    #[serde(default)]
    pub end_points: Vec<BpmnEndPointSnapshot>,
    /// Number of top-level `resource` elements discovered in the document.
    #[serde(default)]
    pub resource_count: usize,
    /// Bounded top-level `resource` metadata preserved from the document.
    #[serde(default)]
    pub resources: Vec<BpmnResourceSnapshot>,
    /// Number of top-level `category` elements discovered in the document.
    #[serde(default)]
    pub category_count: usize,
    /// Bounded top-level `category` metadata preserved from the document.
    #[serde(default)]
    pub categories: Vec<BpmnCategorySnapshot>,
    /// Number of top-level `correlationProperty` elements discovered in the document.
    #[serde(default)]
    pub correlation_property_count: usize,
    /// Bounded top-level `correlationProperty` metadata preserved from the document.
    #[serde(default)]
    pub correlation_properties: Vec<BpmnCorrelationPropertySnapshot>,
    /// Number of top-level `error` elements discovered in the document.
    #[serde(default)]
    pub error_count: usize,
    /// Bounded top-level `error` metadata preserved from the document.
    #[serde(default)]
    pub errors: Vec<BpmnErrorSnapshot>,
    /// Number of top-level `escalation` elements discovered in the document.
    #[serde(default)]
    pub escalation_count: usize,
    /// Bounded top-level `escalation` metadata preserved from the document.
    #[serde(default)]
    pub escalations: Vec<BpmnEscalationSnapshot>,
    /// Number of top-level `signal` elements discovered in the document.
    #[serde(default)]
    pub signal_count: usize,
    /// Bounded top-level `signal` metadata preserved from the document.
    #[serde(default)]
    pub signals: Vec<BpmnSignalSnapshot>,
    /// Number of top-level `dataStore` elements discovered in the document.
    pub data_store_count: usize,
    /// Bounded top-level `dataStore` metadata preserved from the document.
    pub data_stores: Vec<BpmnDataStoreSnapshot>,
    /// Number of top-level `partnerEntity` elements discovered in the document.
    #[serde(default)]
    pub partner_entity_count: usize,
    /// Bounded top-level `partnerEntity` metadata preserved from the document.
    #[serde(default)]
    pub partner_entities: Vec<BpmnPartnerEntitySnapshot>,
    /// Number of top-level `partnerRole` elements discovered in the document.
    #[serde(default)]
    pub partner_role_count: usize,
    /// Bounded top-level `partnerRole` metadata preserved from the document.
    #[serde(default)]
    pub partner_roles: Vec<BpmnPartnerRoleSnapshot>,
    /// Number of top-level global task elements discovered in the document.
    #[serde(default)]
    pub global_task_count: usize,
    /// Bounded top-level global task metadata preserved from the document.
    #[serde(default)]
    pub global_tasks: Vec<BpmnGlobalTaskSnapshot>,
}

pub(crate) fn empty_bpmn_root_snapshot(source: &BpmnSourceFile) -> BpmnRootSnapshot {
    BpmnRootSnapshot {
        element_name: "definitions".to_string(),
        definitions_id: Some(source.source_id.clone()),
        name: None,
        target_namespace: None,
        model_namespace_uri: None,
        import_count: 0,
        imports: Vec::new(),
        extension_count: 0,
        extensions: Vec::new(),
        relationship_count: 0,
        relationships: Vec::new(),
        diagram_count: 0,
        diagrams: Vec::new(),
        collaboration_count: 0,
        process_count: 0,
        item_definition_count: 0,
        item_definitions: Vec::new(),
        message_count: 0,
        messages: Vec::new(),
        interface_count: 0,
        interfaces: Vec::new(),
        end_point_count: 0,
        end_points: Vec::new(),
        resource_count: 0,
        resources: Vec::new(),
        category_count: 0,
        categories: Vec::new(),
        correlation_property_count: 0,
        correlation_properties: Vec::new(),
        error_count: 0,
        errors: Vec::new(),
        escalation_count: 0,
        escalations: Vec::new(),
        signal_count: 0,
        signals: Vec::new(),
        data_store_count: 0,
        data_stores: Vec::new(),
        partner_entity_count: 0,
        partner_entities: Vec::new(),
        partner_role_count: 0,
        partner_roles: Vec::new(),
        global_task_count: 0,
        global_tasks: Vec::new(),
    }
}
