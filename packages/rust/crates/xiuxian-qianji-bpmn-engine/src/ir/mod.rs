//! Internal BPMN IR process/package seam.
//!
//! Public edge, event, index, node, and repeat contracts now live in the
//! crate-root IR owner files. This internal seam stays focused on the process
//! and package shells that tie those public contracts together.

mod process;

pub(crate) use process::{BpmnPackage, BpmnProcessSpec, ProcessKey};
