//! BPMN lint module surface.

mod api;
mod boundary;
mod compensation;
mod document;
mod document_surface;
mod execution;
mod gateway;
mod identity;
mod reference;
mod subprocess;
mod task;
mod topology;
mod transaction;
mod unexpected;

pub(crate) use api::lint_bpmn_source_impl;
