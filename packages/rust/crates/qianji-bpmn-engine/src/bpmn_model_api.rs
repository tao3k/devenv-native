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

/// Snapshot of one BPMN DI `BPMNDiagram`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDiagramSnapshot {
    /// Optional stable BPMN diagram identifier.
    pub diagram_id: Option<String>,
    /// Optional human-readable BPMN diagram name.
    pub name: Option<String>,
    /// Optional BPMN diagram documentation attribute.
    pub documentation: Option<String>,
    /// Optional BPMN diagram resolution attribute.
    pub resolution: Option<String>,
    /// Optional direct nested BPMN DI plane metadata.
    pub plane: Option<BpmnPlaneSnapshot>,
    /// Direct nested BPMN DI label styles preserved in source order.
    #[serde(default)]
    pub label_styles: Vec<BpmnLabelStyleSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNPlane`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnPlaneSnapshot {
    /// Optional stable BPMN plane identifier.
    pub plane_id: Option<String>,
    /// Optional referenced BPMN semantic element.
    pub bpmn_element: Option<String>,
    /// Direct nested BPMN DI shapes preserved in source order.
    #[serde(default)]
    pub shapes: Vec<BpmnShapeSnapshot>,
    /// Direct nested BPMN DI edges preserved in source order.
    #[serde(default)]
    pub edges: Vec<BpmnEdgeSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNShape`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnShapeSnapshot {
    /// Optional stable BPMN shape identifier.
    pub shape_id: Option<String>,
    /// Optional referenced BPMN semantic element.
    pub bpmn_element: Option<String>,
    /// Optional horizontal marker.
    pub is_horizontal: Option<bool>,
    /// Optional expanded marker.
    pub is_expanded: Option<bool>,
    /// Optional marker-visibility marker.
    pub is_marker_visible: Option<bool>,
    /// Optional message-visibility marker.
    pub is_message_visible: Option<bool>,
    /// Optional participant band kind.
    pub participant_band_kind: Option<String>,
    /// Optional choreography activity shape reference.
    pub choreography_activity_shape: Option<String>,
    /// Optional direct nested `dc:Bounds` metadata.
    pub bounds: Option<BpmnBoundsSnapshot>,
    /// Optional direct nested BPMN DI label metadata.
    pub label: Option<BpmnLabelSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNEdge`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnEdgeSnapshot {
    /// Optional stable BPMN edge identifier.
    pub edge_id: Option<String>,
    /// Optional referenced BPMN semantic element.
    pub bpmn_element: Option<String>,
    /// Optional source diagram element reference.
    pub source_element: Option<String>,
    /// Optional target diagram element reference.
    pub target_element: Option<String>,
    /// Optional message visible kind.
    pub message_visible_kind: Option<String>,
    /// Direct nested `di:waypoint` metadata preserved in source order.
    #[serde(default)]
    pub waypoints: Vec<BpmnWaypointSnapshot>,
    /// Optional direct nested BPMN DI label metadata.
    pub label: Option<BpmnLabelSnapshot>,
}

/// Snapshot of one direct nested `dc:Bounds` payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnBoundsSnapshot {
    /// Optional direct `x` payload preserved from `dc:Bounds`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `dc:Bounds`.
    pub y: Option<String>,
    /// Optional direct `width` payload preserved from `dc:Bounds`.
    pub width: Option<String>,
    /// Optional direct `height` payload preserved from `dc:Bounds`.
    pub height: Option<String>,
}

/// Snapshot of one direct nested `di:waypoint` payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnWaypointSnapshot {
    /// Optional direct `x` payload preserved from `di:waypoint`.
    pub x: Option<String>,
    /// Optional direct `y` payload preserved from `di:waypoint`.
    pub y: Option<String>,
}

/// Snapshot of one BPMN DI `BPMNLabel`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLabelSnapshot {
    /// Optional stable BPMN label identifier.
    pub label_id: Option<String>,
    /// Optional referenced BPMN label style.
    pub label_style: Option<String>,
    /// Optional direct nested `dc:Bounds` metadata.
    pub bounds: Option<BpmnBoundsSnapshot>,
}

/// Snapshot of one BPMN DI `BPMNLabelStyle`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLabelStyleSnapshot {
    /// Optional stable BPMN label style identifier.
    pub style_id: Option<String>,
    /// Optional direct nested `dc:Font` metadata.
    pub font: Option<BpmnFontSnapshot>,
}

/// Snapshot of one direct nested `dc:Font` payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFontSnapshot {
    /// Optional font family name.
    pub name: Option<String>,
    /// Optional font size payload.
    pub size: Option<String>,
    /// Optional bold marker.
    pub is_bold: Option<bool>,
    /// Optional italic marker.
    pub is_italic: Option<bool>,
    /// Optional underline marker.
    pub is_underline: Option<bool>,
    /// Optional strike-through marker.
    pub is_strike_through: Option<bool>,
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
    pub task_kind: String,
    /// Optional stable global task identifier.
    pub task_id: Option<String>,
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
    /// Local BPMN collaboration element kind, such as `collaboration`.
    #[serde(default)]
    pub collaboration_kind: String,
    /// Optional stable collaboration identifier.
    pub collaboration_id: Option<String>,
    /// Optional human-readable collaboration name.
    pub name: Option<String>,
    /// Optional BPMN closed-collaboration marker.
    pub is_closed: Option<bool>,
    /// Optional initiating participant for `globalChoreographyTask`.
    pub initiating_participant_ref: Option<String>,
    /// Direct participant metadata preserved from the collaboration.
    pub participants: Vec<BpmnParticipantSnapshot>,
    /// Direct message-flow metadata preserved from the collaboration.
    pub message_flows: Vec<BpmnMessageFlowSnapshot>,
    /// Direct conversation-node metadata preserved from the collaboration.
    #[serde(default)]
    pub conversation_nodes: Vec<BpmnConversationNodeSnapshot>,
    /// Direct conversation-association metadata preserved from the collaboration.
    #[serde(default)]
    pub conversation_associations: Vec<BpmnConversationAssociationSnapshot>,
    /// Direct participant-association metadata preserved from the collaboration.
    #[serde(default)]
    pub participant_associations: Vec<BpmnParticipantAssociationSnapshot>,
    /// Direct message-flow-association metadata preserved from the collaboration.
    #[serde(default)]
    pub message_flow_associations: Vec<BpmnMessageFlowAssociationSnapshot>,
    /// Direct correlation-key metadata preserved from the collaboration.
    #[serde(default)]
    pub correlation_keys: Vec<BpmnCorrelationKeySnapshot>,
    /// Direct choreography references preserved from the collaboration.
    #[serde(default)]
    pub choreography_refs: Vec<String>,
    /// Direct choreography activity metadata preserved from the choreography.
    #[serde(default)]
    pub choreography_activities: Vec<BpmnChoreographyActivitySnapshot>,
    /// Direct conversation-link metadata preserved from the collaboration.
    #[serde(default)]
    pub conversation_links: Vec<BpmnConversationLinkSnapshot>,
    /// Direct artifact associations preserved from the collaboration.
    #[serde(default)]
    pub associations: Vec<BpmnAssociationSnapshot>,
    /// Direct artifact groups preserved from the collaboration.
    #[serde(default)]
    pub groups: Vec<BpmnGroupSnapshot>,
    /// Direct text annotations preserved from the collaboration.
    #[serde(default)]
    pub text_annotations: Vec<BpmnTextAnnotationSnapshot>,
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
    /// Direct nested interface references preserved in source order.
    #[serde(default)]
    pub interface_refs: Vec<String>,
    /// Direct nested endpoint references preserved in source order.
    #[serde(default)]
    pub end_point_refs: Vec<String>,
    /// Optional direct participant multiplicity metadata.
    #[serde(default)]
    pub participant_multiplicity: Option<BpmnParticipantMultiplicitySnapshot>,
}

/// Snapshot of one BPMN `participantMultiplicity`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnParticipantMultiplicitySnapshot {
    /// Optional stable multiplicity identifier.
    pub multiplicity_id: Option<String>,
    /// Optional BPMN minimum payload.
    pub minimum: Option<String>,
    /// Optional BPMN maximum payload.
    pub maximum: Option<String>,
}

/// Snapshot of one BPMN `partnerEntity`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnPartnerEntitySnapshot {
    /// Optional stable partner-entity identifier.
    pub partner_entity_id: Option<String>,
    /// Optional human-readable partner-entity name.
    pub name: Option<String>,
    /// Direct participant references preserved in source order.
    #[serde(default)]
    pub participant_refs: Vec<String>,
}

/// Snapshot of one BPMN `partnerRole`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnPartnerRoleSnapshot {
    /// Optional stable partner-role identifier.
    pub partner_role_id: Option<String>,
    /// Optional human-readable partner-role name.
    pub name: Option<String>,
    /// Direct participant references preserved in source order.
    #[serde(default)]
    pub participant_refs: Vec<String>,
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

/// Snapshot of one BPMN conversation node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnConversationNodeSnapshot {
    /// Local BPMN conversation-node kind.
    pub node_kind: String,
    /// Optional stable conversation-node identifier.
    pub node_id: Option<String>,
    /// Optional human-readable conversation-node name.
    pub name: Option<String>,
    /// Optional called collaboration reference for `callConversation`.
    pub called_collaboration_ref: Option<String>,
    /// Direct participant references preserved in source order.
    #[serde(default)]
    pub participant_refs: Vec<String>,
    /// Direct message-flow references preserved in source order.
    #[serde(default)]
    pub message_flow_refs: Vec<String>,
    /// Direct correlation keys preserved from this conversation node.
    #[serde(default)]
    pub correlation_keys: Vec<BpmnCorrelationKeySnapshot>,
    /// Direct participant associations preserved from this conversation node.
    #[serde(default)]
    pub participant_associations: Vec<BpmnParticipantAssociationSnapshot>,
    /// Direct child conversation nodes preserved from this conversation node.
    #[serde(default)]
    pub child_nodes: Vec<BpmnConversationNodeSnapshot>,
}

/// Snapshot of one BPMN choreography activity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnChoreographyActivitySnapshot {
    /// Local BPMN choreography activity kind.
    pub activity_kind: String,
    /// Optional stable choreography activity identifier.
    pub activity_id: Option<String>,
    /// Optional human-readable choreography activity name.
    pub name: Option<String>,
    /// Optional initiating participant reference.
    pub initiating_participant_ref: Option<String>,
    /// Optional BPMN choreography loop type.
    pub loop_type: Option<String>,
    /// Optional called choreography reference for `callChoreography`.
    pub called_choreography_ref: Option<String>,
    /// Direct participant references preserved in source order.
    #[serde(default)]
    pub participant_refs: Vec<String>,
    /// Direct message-flow references preserved in source order.
    #[serde(default)]
    pub message_flow_refs: Vec<String>,
    /// Direct correlation keys preserved from this choreography activity.
    #[serde(default)]
    pub correlation_keys: Vec<BpmnCorrelationKeySnapshot>,
    /// Direct participant associations preserved from this choreography activity.
    #[serde(default)]
    pub participant_associations: Vec<BpmnParticipantAssociationSnapshot>,
    /// Direct child choreography activities preserved from this activity.
    #[serde(default)]
    pub child_activities: Vec<BpmnChoreographyActivitySnapshot>,
}

/// Snapshot of one BPMN `conversationAssociation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnConversationAssociationSnapshot {
    /// Optional stable conversation-association identifier.
    pub association_id: Option<String>,
    /// Optional inner conversation node reference.
    pub inner_conversation_node_ref: Option<String>,
    /// Optional outer conversation node reference.
    pub outer_conversation_node_ref: Option<String>,
}

/// Snapshot of one BPMN `participantAssociation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnParticipantAssociationSnapshot {
    /// Optional stable participant-association identifier.
    pub association_id: Option<String>,
    /// Optional inner participant reference.
    pub inner_participant_ref: Option<String>,
    /// Optional outer participant reference.
    pub outer_participant_ref: Option<String>,
}

/// Snapshot of one BPMN `messageFlowAssociation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnMessageFlowAssociationSnapshot {
    /// Optional stable message-flow-association identifier.
    pub association_id: Option<String>,
    /// Optional inner message-flow reference.
    pub inner_message_flow_ref: Option<String>,
    /// Optional outer message-flow reference.
    pub outer_message_flow_ref: Option<String>,
}

/// Snapshot of one BPMN `correlationKey`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationKeySnapshot {
    /// Optional stable correlation-key identifier.
    pub correlation_key_id: Option<String>,
    /// Optional human-readable correlation-key name.
    pub name: Option<String>,
    /// Direct correlation-property references preserved in source order.
    #[serde(default)]
    pub correlation_property_refs: Vec<String>,
}

/// Snapshot of one BPMN `conversationLink`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnConversationLinkSnapshot {
    /// Optional stable conversation-link identifier.
    pub link_id: Option<String>,
    /// Optional human-readable conversation-link name.
    pub name: Option<String>,
    /// Optional source reference.
    pub source_ref: Option<String>,
    /// Optional target reference.
    pub target_ref: Option<String>,
}

/// Snapshot of one BPMN artifact `association`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnAssociationSnapshot {
    /// Optional stable association identifier.
    pub association_id: Option<String>,
    /// Optional source reference.
    pub source_ref: Option<String>,
    /// Optional target reference.
    pub target_ref: Option<String>,
    /// Optional BPMN association direction.
    pub association_direction: Option<String>,
}

/// Snapshot of one BPMN artifact `group`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnGroupSnapshot {
    /// Optional stable group identifier.
    pub group_id: Option<String>,
    /// Optional referenced category value.
    pub category_value_ref: Option<String>,
}

/// Snapshot of one BPMN artifact `textAnnotation`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnTextAnnotationSnapshot {
    /// Optional stable text-annotation identifier.
    pub annotation_id: Option<String>,
    /// Optional BPMN text format.
    pub text_format: Option<String>,
    /// Optional nested text payload.
    pub text: Option<String>,
}

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
