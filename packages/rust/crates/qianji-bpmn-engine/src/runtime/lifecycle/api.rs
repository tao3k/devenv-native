//! Internal runtime lifecycle api seam.

pub(crate) use super::advance::apply_pending_host_work_result_impl;
pub(crate) use super::driver::advance_instance_impl;
pub(crate) use super::repeat::merge_output_data;
pub(crate) use super::state::{
    record_transition, resolve_single_outgoing_edge, set_active_node_index, set_node_status,
};
