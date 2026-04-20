//! Internal runtime repeat api seam.

pub(crate) use super::parallel::{
    clear_parallel_multi_instance_state, complete_parallel_multi_instance_iteration,
    has_parallel_multi_instance_state, parallel_multi_instance_state,
    parallel_multi_instance_state_mut, parallel_multi_instance_token_ids,
    register_parallel_multi_instance_iteration,
};
pub(crate) use super::sequential::{
    clear_sequential_multi_instance_state, ensure_sequential_multi_instance_state,
    increment_sequential_multi_instance_iterations, sequential_multi_instance_state,
    sequential_multi_instance_state_mut,
};
pub(crate) use super::standard::{
    clear_standard_loop_state, ensure_standard_loop_state, increment_standard_loop_iterations,
    standard_loop_completed_iterations,
};
pub(crate) use super::state::{
    MultiInstanceCollectionKey, MultiInstanceCollectionKind, MultiInstanceCollectionSlot,
    MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState,
};
pub(crate) use super::variables::{
    parallel_multi_instance_iteration_variables, sequential_multi_instance_iteration_variables,
};
