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
mod identity;
mod loop_risk;
mod reference;
mod subprocess;
mod task;
mod topology;
mod transaction;
mod unexpected;

pub(crate) use api::lint_bpmn_source_impl;
