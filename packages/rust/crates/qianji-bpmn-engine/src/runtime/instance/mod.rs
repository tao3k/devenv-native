//! Internal runtime instance api seam.

mod api;
mod frame;
mod process;
mod repeat;
mod shell;

pub(crate) use api::{
    BpmnInstanceState, EventCompetitionState, InstanceLifecycle, MultiInstanceCollectionKey,
    MultiInstanceCollectionKind, MultiInstanceCollectionSlot, MultiInstanceDataRuntimeState,
    MultiInstanceOutputCollectionState, NodeRuntimeStatus, SuspendReason,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state, complete_parallel_multi_instance_iteration, create_instance_impl,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    has_parallel_multi_instance_state, increment_sequential_multi_instance_iterations,
    increment_standard_loop_iterations, install_process_state,
    parallel_multi_instance_iteration_variables, parallel_multi_instance_state,
    parallel_multi_instance_state_mut, parallel_multi_instance_token_ids, pop_call_activity_frame,
    push_call_activity_frame, register_parallel_multi_instance_iteration,
    resolve_process_for_instance, restore_call_activity_frame,
    sequential_multi_instance_iteration_variables, sequential_multi_instance_state,
    sequential_multi_instance_state_mut, standard_loop_completed_iterations,
};
