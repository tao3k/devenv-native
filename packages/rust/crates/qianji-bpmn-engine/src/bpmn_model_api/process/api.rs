//! Public bpmn model api process contracts for BPMN/DMN engine integration.

use super::artifact::{BpmnAssociationSnapshot, BpmnGroupSnapshot, BpmnTextAnnotationSnapshot};
use super::data::{
    BpmnDataAssociationSnapshot, BpmnDataObjectReferenceSnapshot, BpmnDataObjectSnapshot,
    BpmnDataStoreReferenceSnapshot, BpmnIoBindingSnapshot, BpmnIoSpecificationSnapshot,
};
use super::definitions::BpmnResourceRoleSnapshot;

/// Snapshot of one BPMN `process` metadata shell.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnProcessSnapshot {
    /// Optional stable process identifier.
    pub process_id: Option<String>,
    /// Optional human-readable process name.
    pub name: Option<String>,
    /// Optional BPMN process type marker.
    #[serde(default)]
    pub process_type: Option<String>,
    /// Optional BPMN closed-process marker.
    #[serde(default)]
    pub is_closed: Option<bool>,
    /// Optional BPMN `isExecutable` marker.
    pub is_executable: Option<bool>,
    /// Optional BPMN definitional collaboration reference.
    #[serde(default)]
    pub definitional_collaboration_ref: Option<String>,
    /// Number of direct `supports` references discovered inside this process.
    #[serde(default)]
    pub support_count: usize,
    /// Direct `supports` references preserved from this process.
    #[serde(default)]
    pub supports: Vec<String>,
    /// Number of direct process `property` elements discovered.
    #[serde(default)]
    pub property_count: usize,
    /// Direct process property metadata preserved from this process.
    #[serde(default)]
    pub properties: Vec<BpmnProcessPropertySnapshot>,
    /// Number of direct `correlationSubscription` elements discovered.
    #[serde(default)]
    pub correlation_subscription_count: usize,
    /// Direct process correlation subscriptions preserved from this process.
    #[serde(default)]
    pub correlation_subscriptions: Vec<BpmnCorrelationSubscriptionSnapshot>,
    /// Number of direct resource-role declarations discovered inside this process.
    #[serde(default)]
    pub resource_role_count: usize,
    /// Direct process resource-role metadata preserved from this process.
    #[serde(default)]
    pub resource_roles: Vec<BpmnResourceRoleSnapshot>,
    /// Number of direct process flow elements with common metadata declarations.
    #[serde(default)]
    pub flow_element_metadata_count: usize,
    /// Direct process flow-element common metadata preserved from this process.
    #[serde(default)]
    pub flow_element_metadata: Vec<BpmnFlowElementMetadataSnapshot>,
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
    /// Number of direct process `ioBinding` elements discovered.
    #[serde(default)]
    pub io_binding_count: usize,
    /// Bounded process `ioBinding` metadata preserved from this process.
    #[serde(default)]
    pub io_bindings: Vec<BpmnIoBindingSnapshot>,
    /// Number of `dataInputAssociation` elements discovered inside this process.
    pub data_input_association_count: usize,
    /// Bounded `dataInputAssociation` metadata preserved from this process.
    pub data_input_associations: Vec<BpmnDataAssociationSnapshot>,
    /// Number of `dataOutputAssociation` elements discovered inside this process.
    pub data_output_association_count: usize,
    /// Bounded `dataOutputAssociation` metadata preserved from this process.
    pub data_output_associations: Vec<BpmnDataAssociationSnapshot>,
    /// Number of artifact `association` elements discovered inside this process.
    #[serde(default)]
    pub association_count: usize,
    /// Bounded artifact `association` metadata preserved from this process.
    #[serde(default)]
    pub associations: Vec<BpmnAssociationSnapshot>,
    /// Number of artifact `group` elements discovered inside this process.
    #[serde(default)]
    pub group_count: usize,
    /// Bounded artifact `group` metadata preserved from this process.
    #[serde(default)]
    pub groups: Vec<BpmnGroupSnapshot>,
    /// Number of `textAnnotation` elements discovered inside this process.
    #[serde(default)]
    pub text_annotation_count: usize,
    /// Bounded `textAnnotation` metadata preserved from this process.
    #[serde(default)]
    pub text_annotations: Vec<BpmnTextAnnotationSnapshot>,
}

/// Snapshot of common metadata declared by one direct process `flowElement`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFlowElementMetadataSnapshot {
    /// Local BPMN flow-element kind.
    pub element_kind: String,
    /// Optional stable flow-element identifier.
    pub element_id: Option<String>,
    /// Optional human-readable flow-element name.
    pub name: Option<String>,
    /// Whether this flow element declares direct `auditing` metadata.
    #[serde(default)]
    pub has_auditing: bool,
    /// Optional direct `auditing` identifier.
    #[serde(default)]
    pub auditing_id: Option<String>,
    /// Whether this flow element declares direct `monitoring` metadata.
    #[serde(default)]
    pub has_monitoring: bool,
    /// Optional direct `monitoring` identifier.
    #[serde(default)]
    pub monitoring_id: Option<String>,
    /// Direct `categoryValueRef` values preserved in source order.
    #[serde(default)]
    pub category_value_refs: Vec<String>,
}

/// Snapshot of one direct BPMN process `property`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnProcessPropertySnapshot {
    /// Optional stable process-property identifier.
    pub property_id: Option<String>,
    /// Optional process-property name.
    pub name: Option<String>,
    /// Optional referenced item definition.
    pub item_subject_ref: Option<String>,
}

/// Snapshot of one BPMN process `correlationSubscription`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationSubscriptionSnapshot {
    /// Optional stable subscription identifier.
    pub subscription_id: Option<String>,
    /// Optional referenced correlation key.
    pub correlation_key_ref: Option<String>,
    /// Direct correlation property bindings preserved from this subscription.
    #[serde(default)]
    pub bindings: Vec<BpmnCorrelationPropertyBindingSnapshot>,
}

/// Snapshot of one BPMN `correlationPropertyBinding`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationPropertyBindingSnapshot {
    /// Optional stable binding identifier.
    pub binding_id: Option<String>,
    /// Optional referenced correlation property.
    pub correlation_property_ref: Option<String>,
    /// Optional direct nested `dataPath` payload.
    pub data_path: Option<String>,
    /// Optional formal expression language for `dataPath`.
    pub data_path_language: Option<String>,
    /// Optional formal expression result type reference for `dataPath`.
    pub data_path_evaluates_to_type_ref: Option<String>,
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
