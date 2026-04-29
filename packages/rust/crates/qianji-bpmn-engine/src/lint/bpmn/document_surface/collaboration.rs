//! Canonical `api` entry for collaboration document-surface evidence.

use super::{
    BpmnAssociationSnapshot, BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationNodeSnapshot, BpmnDocumentSnapshot, BpmnGroupSnapshot, BpmnParticipantSnapshot,
    BpmnPartnerEntitySnapshot, BpmnPartnerRoleSnapshot, BpmnTextAnnotationSnapshot,
    CollaborationCounts, SNAPSHOT_EVIDENCE_LIMIT, Value, json,
};

mod api;
mod artifact;
mod choreography;
mod conversation;
mod counts;
mod evidence;
mod participant;
mod root;
mod routing;

use artifact::{artifact_association_evidence, artifact_group_evidence, text_annotation_evidence};
use choreography::{
    choreography_activity_correlation_key_count, choreography_activity_count,
    choreography_activity_evidence,
};
use conversation::{
    collaboration_correlation_key_count, conversation_node_count, conversation_node_evidence,
};
use participant::participant_evidence;

pub(super) use api::{
    collaboration_counts, collaboration_evidence, correlation_property_evidence,
    interface_evidence, item_definition_evidence, message_evidence, partner_entity_evidence,
    partner_role_evidence, routing_boundary_evidence,
};
