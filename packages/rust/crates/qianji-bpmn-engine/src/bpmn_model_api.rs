use crate::bpmn_parse_api::BpmnSourceFile;

/// Snapshot of one BPMN document discovered before executable subset checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDocumentSnapshot {
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Root metadata discovered from the BPMN document.
    pub root: BpmnRootSnapshot,
    /// Top-level collaboration metadata discovered in source order.
    pub collaborations: Vec<BpmnCollaborationSnapshot>,
    /// Top-level process metadata discovered in source order.
    pub processes: Vec<BpmnProcessSnapshot>,
}

impl BpmnDocumentSnapshot {
    /// Returns one process snapshot by id.
    #[must_use]
    pub fn process(&self, process_id: &str) -> Option<&BpmnProcessSnapshot> {
        self.processes
            .iter()
            .find(|process| process.process_id.as_deref() == Some(process_id))
    }
}

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
    /// Number of top-level `relationship` elements discovered in the document.
    #[serde(default)]
    pub relationship_count: usize,
    /// Bounded top-level `relationship` metadata preserved from the document.
    #[serde(default)]
    pub relationships: Vec<BpmnRelationshipSnapshot>,
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
}

/// Snapshot of one BPMN `import`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnImportSnapshot {
    /// Optional imported model namespace.
    pub namespace: Option<String>,
    /// Optional import location.
    pub location: Option<String>,
    /// Optional imported model type URI.
    pub import_type: Option<String>,
}

/// Snapshot of one BPMN `relationship`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnRelationshipSnapshot {
    /// Optional stable relationship identifier.
    pub relationship_id: Option<String>,
    /// Required relationship type preserved as optional metadata for recovery.
    pub relationship_type: Option<String>,
    /// Optional relationship direction.
    pub direction: Option<String>,
    /// Direct source references preserved in source order.
    #[serde(default)]
    pub source_refs: Vec<String>,
    /// Direct target references preserved in source order.
    #[serde(default)]
    pub target_refs: Vec<String>,
}

/// Snapshot of one BPMN `itemDefinition`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnItemDefinitionSnapshot {
    /// Optional stable item-definition identifier.
    pub item_definition_id: Option<String>,
    /// Optional referenced external or model structure.
    pub structure_ref: Option<String>,
    /// Optional BPMN item kind.
    pub item_kind: Option<String>,
    /// Optional BPMN collection marker.
    pub is_collection: Option<bool>,
}

/// Snapshot of one BPMN `message`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnMessageSnapshot {
    /// Optional stable message identifier.
    pub message_id: Option<String>,
    /// Optional human-readable message name.
    pub name: Option<String>,
    /// Optional BPMN item definition reference.
    pub item_ref: Option<String>,
}

/// Snapshot of one BPMN `interface`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnInterfaceSnapshot {
    /// Optional stable interface identifier.
    pub interface_id: Option<String>,
    /// Optional human-readable interface name.
    pub name: Option<String>,
    /// Optional referenced implementation artifact.
    pub implementation_ref: Option<String>,
    /// Direct operation metadata preserved from this interface.
    #[serde(default)]
    pub operations: Vec<BpmnOperationSnapshot>,
}

/// Snapshot of one BPMN `operation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnOperationSnapshot {
    /// Optional stable operation identifier.
    pub operation_id: Option<String>,
    /// Optional human-readable operation name.
    pub name: Option<String>,
    /// Optional referenced implementation artifact.
    pub implementation_ref: Option<String>,
    /// Optional nested input message reference.
    pub in_message_ref: Option<String>,
    /// Optional nested output message reference.
    pub out_message_ref: Option<String>,
    /// Direct nested error references preserved in source order.
    #[serde(default)]
    pub error_refs: Vec<String>,
}

/// Snapshot of one BPMN `resource`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnResourceSnapshot {
    /// Optional stable resource identifier.
    pub resource_id: Option<String>,
    /// Optional human-readable resource name.
    pub name: Option<String>,
    /// Direct resource-parameter metadata preserved from this resource.
    #[serde(default)]
    pub resource_parameters: Vec<BpmnResourceParameterSnapshot>,
}

/// Snapshot of one BPMN `resourceParameter`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnResourceParameterSnapshot {
    /// Optional stable resource-parameter identifier.
    pub resource_parameter_id: Option<String>,
    /// Optional human-readable resource-parameter name.
    pub name: Option<String>,
    /// Optional BPMN type reference.
    pub type_ref: Option<String>,
    /// Optional required-parameter marker.
    pub is_required: Option<bool>,
}

/// Snapshot of one BPMN `category`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCategorySnapshot {
    /// Optional stable category identifier.
    pub category_id: Option<String>,
    /// Optional human-readable category name.
    pub name: Option<String>,
    /// Direct category values preserved from this category.
    #[serde(default)]
    pub category_values: Vec<BpmnCategoryValueSnapshot>,
}

/// Snapshot of one BPMN `categoryValue`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCategoryValueSnapshot {
    /// Optional stable category-value identifier.
    pub category_value_id: Option<String>,
    /// Optional category value payload.
    pub value: Option<String>,
}

/// Snapshot of one BPMN `correlationProperty`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationPropertySnapshot {
    /// Optional stable correlation-property identifier.
    pub correlation_property_id: Option<String>,
    /// Optional human-readable correlation-property name.
    pub name: Option<String>,
    /// Optional BPMN type reference.
    pub type_ref: Option<String>,
    /// Direct retrieval expressions preserved from this correlation property.
    #[serde(default)]
    pub retrieval_expressions: Vec<BpmnCorrelationRetrievalExpressionSnapshot>,
}

/// Snapshot of one BPMN `correlationPropertyRetrievalExpression`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationRetrievalExpressionSnapshot {
    /// Optional stable retrieval-expression identifier.
    pub retrieval_expression_id: Option<String>,
    /// Optional referenced BPMN message identifier.
    pub message_ref: Option<String>,
    /// Optional direct nested `messagePath` payload.
    pub message_path: Option<String>,
}

/// Snapshot of one BPMN `error`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnErrorSnapshot {
    /// Optional stable error identifier.
    pub error_id: Option<String>,
    /// Optional human-readable error name.
    pub name: Option<String>,
    /// Optional BPMN error code.
    pub error_code: Option<String>,
    /// Optional referenced external or model structure.
    pub structure_ref: Option<String>,
}

/// Snapshot of one BPMN `escalation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnEscalationSnapshot {
    /// Optional stable escalation identifier.
    pub escalation_id: Option<String>,
    /// Optional human-readable escalation name.
    pub name: Option<String>,
    /// Optional BPMN escalation code.
    pub escalation_code: Option<String>,
    /// Optional referenced external or model structure.
    pub structure_ref: Option<String>,
}

/// Snapshot of one BPMN `signal`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnSignalSnapshot {
    /// Optional stable signal identifier.
    pub signal_id: Option<String>,
    /// Optional human-readable signal name.
    pub name: Option<String>,
    /// Optional referenced external or model structure.
    pub structure_ref: Option<String>,
}

/// Snapshot of one BPMN `collaboration`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCollaborationSnapshot {
    /// Optional stable collaboration identifier.
    pub collaboration_id: Option<String>,
    /// Optional human-readable collaboration name.
    pub name: Option<String>,
    /// Direct participant metadata preserved from the collaboration.
    pub participants: Vec<BpmnParticipantSnapshot>,
    /// Direct message-flow metadata preserved from the collaboration.
    pub message_flows: Vec<BpmnMessageFlowSnapshot>,
}

/// Snapshot of one BPMN `participant`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnParticipantSnapshot {
    /// Optional stable participant identifier.
    pub participant_id: Option<String>,
    /// Optional human-readable participant name.
    pub name: Option<String>,
    /// Optional referenced process identifier.
    pub process_ref: Option<String>,
}

/// Snapshot of one BPMN `messageFlow`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnMessageFlowSnapshot {
    /// Optional stable message-flow identifier.
    pub message_flow_id: Option<String>,
    /// Optional human-readable message-flow name.
    pub name: Option<String>,
    /// Optional BPMN source reference.
    pub source_ref: Option<String>,
    /// Optional BPMN target reference.
    pub target_ref: Option<String>,
    /// Optional BPMN message reference.
    pub message_ref: Option<String>,
}

/// Snapshot of one BPMN `process` metadata shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnProcessSnapshot {
    /// Optional stable process identifier.
    pub process_id: Option<String>,
    /// Optional human-readable process name.
    pub name: Option<String>,
    /// Optional BPMN `isExecutable` marker.
    pub is_executable: Option<bool>,
    /// Number of `laneSet` elements discovered inside this process.
    pub lane_set_count: usize,
    /// Bounded `laneSet` metadata preserved from this process.
    pub lane_sets: Vec<BpmnLaneSetSnapshot>,
    /// Number of `dataObject` elements discovered inside this process.
    pub data_object_count: usize,
    /// Bounded `dataObject` metadata preserved from this process.
    pub data_objects: Vec<BpmnDataObjectSnapshot>,
    /// Number of `dataObjectReference` elements discovered inside this process.
    pub data_object_reference_count: usize,
    /// Bounded `dataObjectReference` metadata preserved from this process.
    pub data_object_references: Vec<BpmnDataObjectReferenceSnapshot>,
    /// Number of `dataStoreReference` elements discovered inside this process.
    pub data_store_reference_count: usize,
    /// Bounded `dataStoreReference` metadata preserved from this process.
    pub data_store_references: Vec<BpmnDataStoreReferenceSnapshot>,
    /// Number of `ioSpecification` elements discovered inside this process.
    pub io_specification_count: usize,
    /// Bounded `ioSpecification` metadata preserved from this process.
    pub io_specifications: Vec<BpmnIoSpecificationSnapshot>,
    /// Number of `dataInputAssociation` elements discovered inside this process.
    pub data_input_association_count: usize,
    /// Bounded `dataInputAssociation` metadata preserved from this process.
    pub data_input_associations: Vec<BpmnDataAssociationSnapshot>,
    /// Number of `dataOutputAssociation` elements discovered inside this process.
    pub data_output_association_count: usize,
    /// Bounded `dataOutputAssociation` metadata preserved from this process.
    pub data_output_associations: Vec<BpmnDataAssociationSnapshot>,
}

/// Snapshot of one BPMN `laneSet`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLaneSetSnapshot {
    /// Optional stable lane-set identifier.
    pub lane_set_id: Option<String>,
    /// Optional human-readable lane-set name.
    pub name: Option<String>,
    /// Direct lane metadata preserved from this lane set.
    pub lanes: Vec<BpmnLaneSnapshot>,
}

/// Snapshot of one BPMN `lane`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLaneSnapshot {
    /// Optional stable lane identifier.
    pub lane_id: Option<String>,
    /// Optional human-readable lane name.
    pub name: Option<String>,
    /// Direct `flowNodeRef` payloads preserved in source order.
    pub flow_node_refs: Vec<String>,
}

/// Snapshot of one BPMN `dataObject`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataObjectSnapshot {
    /// Optional stable data-object identifier.
    pub data_object_id: Option<String>,
    /// Optional human-readable data-object name.
    pub name: Option<String>,
    /// Optional BPMN item-subject reference.
    pub item_subject_ref: Option<String>,
    /// Optional BPMN collection marker.
    pub is_collection: Option<bool>,
}

/// Snapshot of one BPMN `dataObjectReference`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataObjectReferenceSnapshot {
    /// Optional stable data-object-reference identifier.
    pub data_object_reference_id: Option<String>,
    /// Optional human-readable data-object-reference name.
    pub name: Option<String>,
    /// Optional referenced `dataObject` identifier.
    pub data_object_ref: Option<String>,
}

/// Snapshot of one BPMN `dataStore`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataStoreSnapshot {
    /// Optional stable data-store identifier.
    pub data_store_id: Option<String>,
    /// Optional human-readable data-store name.
    pub name: Option<String>,
    /// Optional BPMN item-subject reference.
    pub item_subject_ref: Option<String>,
    /// Optional BPMN capacity payload.
    pub capacity: Option<String>,
    /// Optional BPMN unlimited-capacity marker.
    pub is_unlimited: Option<bool>,
}

/// Snapshot of one BPMN `dataStoreReference`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataStoreReferenceSnapshot {
    /// Optional stable data-store-reference identifier.
    pub data_store_reference_id: Option<String>,
    /// Optional human-readable data-store-reference name.
    pub name: Option<String>,
    /// Optional referenced `dataStore` identifier.
    pub data_store_ref: Option<String>,
}

/// Snapshot of one BPMN `ioSpecification`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnIoSpecificationSnapshot {
    /// Optional stable IO-specification identifier.
    pub io_specification_id: Option<String>,
    /// Direct data-input metadata preserved from this IO specification.
    pub data_inputs: Vec<BpmnDataInputOutputSnapshot>,
    /// Direct data-output metadata preserved from this IO specification.
    pub data_outputs: Vec<BpmnDataInputOutputSnapshot>,
}

/// Snapshot of one BPMN `dataInput` or `dataOutput`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataInputOutputSnapshot {
    /// Optional stable data input/output identifier.
    pub data_id: Option<String>,
    /// Optional human-readable input/output name.
    pub name: Option<String>,
    /// Optional BPMN item-subject reference.
    pub item_subject_ref: Option<String>,
    /// Optional BPMN collection marker.
    pub is_collection: Option<bool>,
}

/// Snapshot of one BPMN data association.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataAssociationSnapshot {
    /// Optional stable data-association identifier.
    pub association_id: Option<String>,
    /// Direct nested `sourceRef` payloads preserved in source order.
    pub source_refs: Vec<String>,
    /// Optional direct nested `targetRef` payload.
    pub target_ref: Option<String>,
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
        relationship_count: 0,
        relationships: Vec::new(),
        collaboration_count: 0,
        process_count: 0,
        item_definition_count: 0,
        item_definitions: Vec::new(),
        message_count: 0,
        messages: Vec::new(),
        interface_count: 0,
        interfaces: Vec::new(),
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
    }
}
