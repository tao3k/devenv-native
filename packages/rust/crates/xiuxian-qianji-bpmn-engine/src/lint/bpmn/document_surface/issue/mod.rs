//! Canonical API seam for deferred BPMN document-surface issues.

mod api;
mod collaboration;
mod data;
mod diagram;
mod dispatch;
mod metadata;

pub(super) use api::{flow_element_metadata_issue, issue_for_tag, resource_role_metadata_issue};
