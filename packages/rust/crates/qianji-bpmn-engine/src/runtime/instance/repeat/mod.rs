//! Internal runtime repeat api seam.

mod api;
mod parallel;
mod sequential;
mod standard;
mod state;
mod variables;

pub(crate) use api::{
    MultiInstanceCollectionKey, MultiInstanceCollectionKind, MultiInstanceCollectionSlot,
    MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state, complete_parallel_multi_instance_iteration,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    has_parallel_multi_instance_state, increment_sequential_multi_instance_iterations,
    increment_standard_loop_iterations, parallel_multi_instance_iteration_variables,
    parallel_multi_instance_min_token_id, parallel_multi_instance_state,
    parallel_multi_instance_state_mut, register_parallel_multi_instance_iteration,
    sequential_multi_instance_iteration_variables, sequential_multi_instance_state,
    sequential_multi_instance_state_mut, standard_loop_completed_iterations,
};
