//! Canonical api seam for BPMN document-surface lint checks.

use crate::bpmn_model_api::{
    BpmnAssociationSnapshot, BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationNodeSnapshot, BpmnDataAssociationExpressionSnapshot,
    BpmnDataAssociationSnapshot, BpmnDataStateSnapshot, BpmnDocumentSnapshot,
    BpmnFlowElementMetadataSnapshot, BpmnGlobalTaskSnapshot, BpmnGroupSnapshot,
    BpmnInputSetSnapshot, BpmnIoBindingSnapshot, BpmnOutputSetSnapshot, BpmnParticipantSnapshot,
    BpmnPartnerEntitySnapshot, BpmnPartnerRoleSnapshot, BpmnProcessSnapshot,
    BpmnResourceRoleSnapshot, BpmnTextAnnotationSnapshot,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint_api::LintIssue;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Value, json};

const SNAPSHOT_EVIDENCE_LIMIT: usize = 8;

mod api;
mod collaboration;
mod data;
mod issue;
mod metadata;
mod model;
mod summary;
mod xml;

use collaboration::{
    collaboration_counts, collaboration_evidence, correlation_property_evidence,
    interface_evidence, item_definition_evidence, message_evidence, partner_entity_evidence,
    partner_role_evidence, routing_boundary_evidence,
};
use data::data_snapshot_summary;
use issue::issue_for_tag;
use metadata::{flow_element_metadata_summary, process_callable_summary, resource_role_summary};
use model::{
    CollaborationCounts, FlowElementMetadataCounts, ProcessCallableCounts, ResourceRoleCounts,
};
use summary::{document_surface_evidence, root_snapshot_summary};
use xml::local_name;

pub(super) use api::deferred_document_surface_issue;
