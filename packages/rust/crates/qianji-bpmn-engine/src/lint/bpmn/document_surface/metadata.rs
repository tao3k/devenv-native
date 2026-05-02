//! Canonical `api` entry for metadata document-surface summaries.

use super::{
    FlowElementMetadataCounts, ProcessCallableCounts, ResourceRoleCounts, SNAPSHOT_EVIDENCE_LIMIT,
};
use crate::bpmn_model_api::{
    BpmnDocumentSnapshot, BpmnFlowElementMetadataSnapshot, BpmnGlobalTaskSnapshot,
    BpmnIoBindingSnapshot, BpmnProcessSnapshot, BpmnResourceRoleSnapshot,
};
use serde_json::{Value, json};

mod api;
mod callable;
mod correlation;
mod flow;
mod resource;

use correlation::correlation_boundary_evidence;

pub(super) use api::{
    flow_element_metadata_summary, process_callable_summary, resource_role_summary,
};
