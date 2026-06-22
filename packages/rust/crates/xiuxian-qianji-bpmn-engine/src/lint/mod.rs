//! Internal lint api seam.

mod api;
mod bpmn;
mod dmn;

pub(crate) use api::{lint_bpmn_source_impl, lint_dmn_source_impl};
