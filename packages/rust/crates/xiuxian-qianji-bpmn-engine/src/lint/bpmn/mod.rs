//! BPMN lint module surface.

mod api;
mod boundary;
mod compensation;
mod condition_contract;
mod data_contract;
mod document;
mod document_surface;
mod execution;
mod extension;
mod gateway;
mod human_task;
mod identity;
mod loop_risk;
mod reference;
mod subprocess;
mod task;
mod task_binding;
mod topology;
mod transaction;
mod unexpected;

pub(crate) use api::lint_bpmn_source_impl;
pub(in crate::lint::bpmn) use condition_contract::{
    ambiguous_boolean_gateway_condition_issues, ambiguous_boolean_gateway_condition_source_issues,
    unsupported_gateway_condition_source_issues,
};
pub(in crate::lint::bpmn) use data_contract::undeclared_gateway_condition_output_issues;
pub(in crate::lint::bpmn) use document::issue_from_bpmn_document_error;
pub(in crate::lint::bpmn) use document_surface::deferred_document_surface_issue;
pub(in crate::lint::bpmn) use execution::issue_from_bpmn_execution_shape_error;
pub(in crate::lint::bpmn) use extension::human_task_interaction_issues;
pub(in crate::lint::bpmn) use human_task::{
    human_task_standard_issues, issue_from_bpmn_human_task_standard_error,
};
pub(in crate::lint::bpmn) use identity::issue_from_bpmn_identity_error;
pub(in crate::lint::bpmn) use loop_risk::loop_risk_issues;
pub(in crate::lint::bpmn) use reference::issue_from_bpmn_reference_error;
pub(in crate::lint::bpmn) use task_binding::task_operation_binding_issues;
pub(in crate::lint::bpmn) use topology::issue_from_bpmn_topology_error;
pub(in crate::lint::bpmn) use unexpected::unexpected_bpmn_issue;
