//! Public bpmn model api definitions contracts for BPMN/DMN engine integration.

use super::data::{BpmnIoBindingSnapshot, BpmnIoSpecificationSnapshot};
use super::types::{BpmnSnapshotFlag, BpmnSnapshotId, BpmnSnapshotKind, BpmnSnapshotType};

/// Snapshot of one BPMN `import`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnImportSnapshot {
    /// Optional imported model namespace.
    pub namespace: Option<String>,
    /// Optional import location.
    pub location: Option<String>,
    /// Optional imported model type URI.
    pub import_type: Option<BpmnSnapshotType>,
}

/// Snapshot of one BPMN `extension` declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnExtensionSnapshot {
    /// Optional extension definition `QName`.
    pub definition: Option<String>,
    /// Resolved BPMN `mustUnderstand` marker; absent attributes default to `false`.
    #[serde(default)]
    pub must_understand: bool,
    /// Direct documentation text values preserved in source order.
    #[serde(default)]
    pub documentation: Vec<String>,
}

/// Snapshot of one BPMN `relationship`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnRelationshipSnapshot {
    /// Optional stable relationship identifier.
    pub relationship_id: Option<BpmnSnapshotId>,
    /// Required relationship type preserved as optional metadata for recovery.
    pub relationship_type: Option<BpmnSnapshotType>,
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
    pub item_definition_id: Option<BpmnSnapshotId>,
    /// Optional referenced external or model structure.
    pub structure_ref: Option<String>,
    /// Optional BPMN item kind.
    pub item_kind: Option<BpmnSnapshotKind>,
    /// Optional BPMN collection marker.
    pub is_collection: Option<BpmnSnapshotFlag>,
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

/// Snapshot of one BPMN `endPoint`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnEndPointSnapshot {
    /// Optional stable endpoint identifier.
    pub end_point_id: Option<String>,
}

/// Snapshot of one top-level BPMN global task definition.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnGlobalTaskSnapshot {
    /// Local BPMN global task kind.
    pub task_kind: BpmnSnapshotKind,
    /// Optional stable global task identifier.
    pub task_id: Option<BpmnSnapshotId>,
    /// Optional human-readable global task name.
    pub name: Option<String>,
    /// Optional BPMN implementation marker.
    pub implementation: Option<String>,
    /// Optional BPMN script language marker for `globalScriptTask`.
    pub script_language: Option<String>,
    /// Optional direct script payload for `globalScriptTask`.
    pub script: Option<String>,
    /// Direct supported interface references preserved in source order.
    #[serde(default)]
    pub supported_interface_refs: Vec<String>,
    /// Number of direct `ioSpecification` elements discovered on this global task.
    #[serde(default)]
    pub io_specification_count: usize,
    /// Direct global-task `ioSpecification` metadata preserved from this global task.
    #[serde(default)]
    pub io_specifications: Vec<BpmnIoSpecificationSnapshot>,
    /// Number of direct `ioBinding` elements discovered on this global task.
    #[serde(default)]
    pub io_binding_count: usize,
    /// Direct global-task `ioBinding` metadata preserved from this global task.
    #[serde(default)]
    pub io_bindings: Vec<BpmnIoBindingSnapshot>,
    /// Number of direct resource-role declarations discovered on this global task.
    #[serde(default)]
    pub resource_role_count: usize,
    /// Direct resource-role metadata preserved from this global task.
    #[serde(default)]
    pub resource_roles: Vec<BpmnResourceRoleSnapshot>,
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

/// Snapshot of one BPMN resource-role declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnResourceRoleSnapshot {
    /// Local BPMN resource-role kind.
    pub role_kind: BpmnSnapshotKind,
    /// Optional stable role identifier.
    pub role_id: Option<BpmnSnapshotId>,
    /// Optional role name.
    pub name: Option<String>,
    /// Optional nested `resourceRef` payload.
    pub resource_ref: Option<String>,
    /// Optional stable `resourceAssignmentExpression` identifier.
    #[serde(default)]
    pub assignment_expression_id: Option<String>,
    /// Optional nested resource assignment expression payload.
    #[serde(default)]
    pub assignment_expression: Option<String>,
    /// Optional formal expression language for the assignment expression.
    #[serde(default)]
    pub assignment_expression_language: Option<String>,
    /// Optional formal expression result type for the assignment expression.
    #[serde(default)]
    pub assignment_expression_evaluates_to_type_ref: Option<String>,
    /// Direct resource-parameter bindings preserved from this role.
    #[serde(default)]
    pub parameter_bindings: Vec<BpmnResourceParameterBindingSnapshot>,
}

/// Snapshot of one BPMN `resourceParameterBinding`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnResourceParameterBindingSnapshot {
    /// Optional stable binding identifier.
    pub binding_id: Option<String>,
    /// Optional referenced resource parameter.
    pub parameter_ref: Option<String>,
    /// Optional nested binding expression payload.
    pub expression: Option<String>,
    /// Optional formal expression language for the binding expression.
    #[serde(default)]
    pub expression_language: Option<String>,
    /// Optional formal expression result type for the binding expression.
    #[serde(default)]
    pub expression_evaluates_to_type_ref: Option<String>,
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
