//! BPMN normalization into immutable runtime-ready IR.

mod compensation;
mod digest;
mod event;
mod index;
mod node;
mod process;
mod repeat;

pub(in crate::parser::normalize) use index::normalize_node_index;
pub(crate) use process::normalize_package;
