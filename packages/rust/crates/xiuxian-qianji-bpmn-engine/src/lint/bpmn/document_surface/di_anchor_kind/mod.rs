//! BPMN DI semantic-anchor kind audit.

mod api;
mod model;

pub(in crate::lint::bpmn::document_surface) use api::diagram_anchor_kind_issue;
