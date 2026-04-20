//! Internal runtime instance api seam.

pub(crate) use super::frame::{
    install_process_state, pop_call_activity_frame, push_call_activity_frame,
    restore_call_activity_frame,
};
pub(crate) use super::process::resolve_process_for_instance;
pub(crate) use super::repeat::{
    MultiInstanceCollectionKey, MultiInstanceCollectionKind, MultiInstanceCollectionSlot,
    MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state, complete_parallel_multi_instance_iteration,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    has_parallel_multi_instance_state, increment_sequential_multi_instance_iterations,
    increment_standard_loop_iterations, parallel_multi_instance_iteration_variables,
    parallel_multi_instance_state, parallel_multi_instance_state_mut,
    parallel_multi_instance_token_ids, register_parallel_multi_instance_iteration,
    sequential_multi_instance_iteration_variables, sequential_multi_instance_state,
    sequential_multi_instance_state_mut, standard_loop_completed_iterations,
};
pub(crate) use super::shell::{
    BpmnInstanceState, EventCompetitionState, InstanceLifecycle, NodeRuntimeStatus, SuspendReason,
    create_instance_impl,
};
