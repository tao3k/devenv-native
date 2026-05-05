//! parser import process branch wiring for focused BPMN/DMN owner leaves.

#[path = "process_child.rs"]
mod child;
#[path = "process_scope.rs"]
mod scope;

pub(super) use child::{handle_process_child_start_tag, is_supported_node_tag};
pub(super) use scope::{complete_process_scope, handle_package_start_tag};
