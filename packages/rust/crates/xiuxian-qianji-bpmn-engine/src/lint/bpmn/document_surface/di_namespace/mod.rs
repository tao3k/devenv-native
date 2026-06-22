//! BPMN DI namespace audit for native diagram interchange.

mod api;
mod model;
mod scan;

pub(in crate::lint::bpmn::document_surface) use api::diagram_namespace_issue;
