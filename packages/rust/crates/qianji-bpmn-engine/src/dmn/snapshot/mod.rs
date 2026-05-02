//! dmn snapshot branch wiring for focused BPMN/DMN owner leaves.

mod root;
mod scan;
mod state;
mod xml;

pub(crate) use scan::snapshot_dmn_source_sync;
pub(in crate::dmn::snapshot) use xml::{attribute_value, required_attribute};
