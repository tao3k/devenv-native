//! Parser import task I/O branch wiring.

mod binding;
mod core;
mod declaration;
mod start;
mod state;

pub(in crate::parser::import) use core::{
    apply_task_io_assignment_from, apply_task_io_assignment_to, apply_task_io_source_ref,
    apply_task_io_target_ref, complete_task_io_end_tag, record_task_io_property_id,
};
pub(super) use start::handle_task_io_child_start;
