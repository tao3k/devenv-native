//! bpmn snapshot branch wiring for focused BPMN/DMN owner leaves.

mod scan;
mod state;
mod xml;

pub(crate) use scan::snapshot_bpmn_source_sync;
pub(in crate::bpmn_snapshot) use xml::{
    attribute_value, boolean_attribute_value, bpmn_model_namespace, local_name,
};
