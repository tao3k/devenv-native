//! Canonical `api` entry for metadata document-surface summaries.

use super::{
    BpmnDocumentSnapshot, BpmnFlowElementMetadataSnapshot, BpmnGlobalTaskSnapshot,
    BpmnIoBindingSnapshot, BpmnProcessSnapshot, BpmnResourceRoleSnapshot,
    FlowElementMetadataCounts, ProcessCallableCounts, ResourceRoleCounts, SNAPSHOT_EVIDENCE_LIMIT,
    Value, json,
};

mod api;
mod callable;
mod flow;
mod resource;

pub(super) use api::{
    flow_element_metadata_summary, process_callable_summary, resource_role_summary,
};
