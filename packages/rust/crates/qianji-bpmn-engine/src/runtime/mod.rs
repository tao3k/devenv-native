//! Internal runtime api seam.

mod api;
mod frontier;
mod host;
mod instance;
mod lifecycle;
mod wait;

pub(crate) use api::{
    BpmnAdvanceOutcome, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierParallelJoinMerge, BpmnFrontierRuntimeAction, BpmnFrontierRuntimeBatch,
    BpmnInstanceState, EventCompetitionState, InstanceLifecycle, JoinRuntimeState,
    MultiInstanceCollectionKey, MultiInstanceCollectionKind, MultiInstanceCollectionSlot,
    MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState, NodeRuntimeStatus,
    PendingHostWork, PendingHostWorkKind, SuspendReason, TokenRecord, WaitKind, WaitRegistration,
    advance_instance_impl, apply_event_poll_outcome, apply_pending_host_work_result,
    build_event_poll_request, build_pending_host_work_request, build_pending_host_work_requests,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state, complete_parallel_multi_instance_iteration, create_instance_impl,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    has_parallel_multi_instance_state, increment_sequential_multi_instance_iterations,
    increment_standard_loop_iterations, install_process_state,
    parallel_multi_instance_iteration_variables, parallel_multi_instance_min_token_id,
    parallel_multi_instance_state, parallel_multi_instance_state_mut, plan_frontier_runtime_action,
    pop_call_activity_frame, push_active_token, push_call_activity_frame,
    register_parallel_multi_instance_iteration, resolve_process_for_instance,
    restore_call_activity_frame, sequential_multi_instance_iteration_variables,
    sequential_multi_instance_state, sequential_multi_instance_state_mut,
    standard_loop_completed_iterations,
};
