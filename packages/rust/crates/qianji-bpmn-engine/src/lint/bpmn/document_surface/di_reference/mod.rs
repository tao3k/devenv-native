//! BPMN DI reference audit for metadata-only diagram interchange surfaces.

mod api;
mod local;
mod model;
mod semantic;

pub(super) use api::diagram_reference_issue;
