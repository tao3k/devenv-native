//! Canonical API seam for deferred BPMN document-surface issues.

mod api;
mod data;
mod dispatch;
mod metadata;

pub(super) use api::{
    flow_element_metadata_issue, io_set_lifecycle_issue, issue_for_tag,
    resource_role_metadata_issue,
};
