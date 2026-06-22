//! BPMN package validation branch wiring.

mod compensation;
mod core;
mod routing;
mod topology;

pub(crate) use core::validate_raw_package;
pub(crate) use routing::resolve_structured_inclusive_join;
