//! Canonical `api` entry for metadata document-surface summaries.

use super::shared::{
    BpmnDocumentSnapshot, BpmnFlowElementMetadataSnapshot, BpmnGlobalTaskSnapshot,
    BpmnIoBindingSnapshot, BpmnProcessSnapshot, BpmnResourceRoleSnapshot,
    FlowElementMetadataCounts, ProcessCallableCounts, ResourceRoleCounts, SNAPSHOT_EVIDENCE_LIMIT,
    Value, json,
};

mod api;
mod callable;
mod correlation;
mod flow;
mod resource;

use correlation::correlation_boundary_evidence;

pub(super) use api::{
    flow_element_metadata_summary, process_callable_summary, resource_role_summary,
};
