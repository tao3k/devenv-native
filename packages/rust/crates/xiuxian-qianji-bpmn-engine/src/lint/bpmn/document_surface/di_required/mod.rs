//! lint bpmn document surface di required branch wiring for focused BPMN/DMN owner leaves.

mod api;
mod model;
mod scan;

pub(super) use api::diagram_required_attribute_issue;
