//! Internal runtime lifecycle api seam.

mod advance;
mod api;
mod blocking;
mod boundary;
mod call_activity;
mod completion;
mod conditional;
mod driver;
mod error;
mod escalation;
mod event_subprocess;
mod gateway;
mod prepare;
mod repeat;
mod scope;
mod state;
mod terminate;
mod transaction;

pub(crate) use api::{
    advance_instance_impl, apply_current_frame_event_subprocess_wait,
    apply_parent_frame_event_subprocess_wait, apply_pending_host_work_result_impl,
    cancel_attached_boundary_siblings, is_event_subprocess_wait, push_active_token,
};
pub(super) use api::{
    conditional_event_is_satisfied, merge_output_data, record_human_task_lifecycle_event,
    record_transition, resolve_single_outgoing_edge, set_active_node_index, set_node_status,
};
