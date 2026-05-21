//! Public bpmn model api data contracts for BPMN/DMN engine integration.

use super::types::{BpmnSnapshotFlag, BpmnSnapshotId};

/// Snapshot of one BPMN `dataObject`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataObjectSnapshot {
    /// Optional stable data-object identifier.
    pub data_object_id: Option<BpmnSnapshotId>,
    /// Optional human-readable data-object name.
    pub name: Option<String>,
    /// Optional BPMN item-subject reference.
    pub item_subject_ref: Option<String>,
    /// Optional BPMN collection marker.
    pub is_collection: Option<BpmnSnapshotFlag>,
    /// Optional direct `dataState` metadata.
    #[serde(default)]
    pub data_state: Option<BpmnDataStateSnapshot>,
}

/// Snapshot of one BPMN `dataObjectReference`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataObjectReferenceSnapshot {
    /// Optional stable data-object-reference identifier.
    pub data_object_reference_id: Option<BpmnSnapshotId>,
    /// Optional human-readable data-object-reference name.
    pub name: Option<String>,
    /// Optional referenced `dataObject` identifier.
    pub data_object_ref: Option<String>,
    /// Optional BPMN item-subject reference.
    #[serde(default)]
    pub item_subject_ref: Option<String>,
    /// Optional direct `dataState` metadata.
    #[serde(default)]
    pub data_state: Option<BpmnDataStateSnapshot>,
}

/// Snapshot of one BPMN `dataStore`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataStoreSnapshot {
    /// Optional stable data-store identifier.
    pub data_store_id: Option<BpmnSnapshotId>,
    /// Optional human-readable data-store name.
    pub name: Option<String>,
    /// Optional BPMN item-subject reference.
    pub item_subject_ref: Option<String>,
    /// Optional BPMN capacity payload.
    pub capacity: Option<String>,
    /// Optional BPMN unlimited-capacity marker.
    pub is_unlimited: Option<BpmnSnapshotFlag>,
    /// Optional direct `dataState` metadata.
    #[serde(default)]
    pub data_state: Option<BpmnDataStateSnapshot>,
}

/// Snapshot of one BPMN `dataStoreReference`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataStoreReferenceSnapshot {
    /// Optional stable data-store-reference identifier.
    pub data_store_reference_id: Option<BpmnSnapshotId>,
    /// Optional human-readable data-store-reference name.
    pub name: Option<String>,
    /// Optional referenced `dataStore` identifier.
    pub data_store_ref: Option<String>,
    /// Optional BPMN item-subject reference.
    #[serde(default)]
    pub item_subject_ref: Option<String>,
    /// Optional direct `dataState` metadata.
    #[serde(default)]
    pub data_state: Option<BpmnDataStateSnapshot>,
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
    /// Direct input-set metadata preserved from this IO specification.
    #[serde(default)]
    pub input_sets: Vec<BpmnInputSetSnapshot>,
    /// Direct output-set metadata preserved from this IO specification.
    #[serde(default)]
    pub output_sets: Vec<BpmnOutputSetSnapshot>,
}

/// Snapshot of one BPMN `inputSet`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnInputSetSnapshot {
    /// Optional stable input-set identifier.
    pub set_id: Option<String>,
    /// Optional human-readable input-set name.
    pub name: Option<String>,
    /// Direct `dataInputRefs` payloads preserved in source order.
    #[serde(default)]
    pub data_input_refs: Vec<String>,
    /// Direct `optionalInputRefs` payloads preserved in source order.
    #[serde(default)]
    pub optional_input_refs: Vec<String>,
    /// Direct `whileExecutingInputRefs` payloads preserved in source order.
    #[serde(default)]
    pub while_executing_input_refs: Vec<String>,
    /// Direct `outputSetRefs` payloads preserved in source order.
    #[serde(default)]
    pub output_set_refs: Vec<String>,
}

/// Snapshot of one BPMN `outputSet`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnOutputSetSnapshot {
    /// Optional stable output-set identifier.
    pub set_id: Option<String>,
    /// Optional human-readable output-set name.
    pub name: Option<String>,
    /// Direct `dataOutputRefs` payloads preserved in source order.
    #[serde(default)]
    pub data_output_refs: Vec<String>,
    /// Direct `optionalOutputRefs` payloads preserved in source order.
    #[serde(default)]
    pub optional_output_refs: Vec<String>,
    /// Direct `whileExecutingOutputRefs` payloads preserved in source order.
    #[serde(default)]
    pub while_executing_output_refs: Vec<String>,
    /// Direct `inputSetRefs` payloads preserved in source order.
    #[serde(default)]
    pub input_set_refs: Vec<String>,
}

/// Snapshot of one BPMN `dataInput` or `dataOutput`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataInputOutputSnapshot {
    /// Optional stable data input/output identifier.
    pub data_id: Option<BpmnSnapshotId>,
    /// Optional human-readable input/output name.
    pub name: Option<String>,
    /// Optional BPMN item-subject reference.
    pub item_subject_ref: Option<String>,
    /// Optional BPMN collection marker.
    pub is_collection: Option<BpmnSnapshotFlag>,
    /// Optional direct `dataState` metadata.
    #[serde(default)]
    pub data_state: Option<BpmnDataStateSnapshot>,
}

/// Snapshot of one BPMN `dataState`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataStateSnapshot {
    /// Optional stable data-state identifier.
    pub data_state_id: Option<String>,
    /// Optional human-readable data-state name.
    pub name: Option<String>,
}

/// Snapshot of one BPMN callable `ioBinding`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnIoBindingSnapshot {
    /// Optional stable IO-binding identifier.
    pub binding_id: Option<String>,
    /// Referenced callable operation identifier.
    pub operation_ref: Option<String>,
    /// Referenced input data identifier.
    pub input_data_ref: Option<String>,
    /// Referenced output data identifier.
    pub output_data_ref: Option<String>,
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
    /// Optional direct `transformation` expression metadata.
    #[serde(default)]
    pub transformation: Option<BpmnDataAssociationExpressionSnapshot>,
    /// Direct nested `assignment` metadata preserved in source order.
    #[serde(default)]
    pub assignments: Vec<BpmnDataAssociationAssignmentSnapshot>,
}

/// Snapshot of one BPMN data-association expression payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataAssociationExpressionSnapshot {
    /// Optional stable expression identifier.
    pub expression_id: Option<String>,
    /// Optional expression text payload.
    #[serde(default)]
    pub body: Option<String>,
    /// Optional formal expression language metadata.
    pub language: Option<String>,
    /// Optional formal expression result type metadata.
    pub evaluates_to_type_ref: Option<String>,
}

/// Snapshot of one BPMN data-association assignment.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataAssociationAssignmentSnapshot {
    /// Optional stable assignment identifier.
    pub assignment_id: Option<String>,
    /// Optional nested `from` expression metadata.
    #[serde(default)]
    pub from: Option<BpmnDataAssociationExpressionSnapshot>,
    /// Optional nested `to` expression metadata.
    #[serde(default)]
    pub to: Option<BpmnDataAssociationExpressionSnapshot>,
}
