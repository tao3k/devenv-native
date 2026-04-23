//! BPMN normalization into immutable runtime-ready IR.

mod compensation;
mod digest;
mod event;
mod node;
mod process;
mod repeat;

pub(crate) use process::normalize_package;
