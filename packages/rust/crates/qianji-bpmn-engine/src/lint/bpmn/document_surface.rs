//! Canonical api seam for BPMN document-surface lint checks.

use crate::bpmn_model_api::{
    BpmnAssociationSnapshot, BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationNodeSnapshot, BpmnDocumentSnapshot, BpmnFlowElementMetadataSnapshot,
    BpmnGlobalTaskSnapshot, BpmnGroupSnapshot, BpmnIoBindingSnapshot, BpmnParticipantSnapshot,
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
mod di_anchor;
mod di_anchor_kind;
mod di_completeness;
mod di_identity;
mod di_reference;
mod di_semantic;
mod di_topology;
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
use di_anchor::diagram_anchor_issue;
use di_anchor_kind::diagram_anchor_kind_issue;
use di_completeness::diagram_completeness_issue;
use di_identity::diagram_identity_issue;
use di_reference::diagram_reference_issue;
use di_topology::diagram_topology_issue;
use issue::{flow_element_metadata_issue, issue_for_tag, resource_role_metadata_issue};
use metadata::{flow_element_metadata_summary, process_callable_summary, resource_role_summary};
use model::{
    CollaborationCounts, FlowElementMetadataCounts, ProcessCallableCounts, ResourceRoleCounts,
};
use xml::local_name;

pub(super) use api::deferred_document_surface_issue;
