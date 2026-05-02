//! Public bpmn model api collaboration contracts for BPMN/DMN engine integration.

use super::artifact::{BpmnAssociationSnapshot, BpmnGroupSnapshot, BpmnTextAnnotationSnapshot};

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
