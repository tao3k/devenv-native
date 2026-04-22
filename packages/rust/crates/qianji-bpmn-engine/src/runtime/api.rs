//! Internal runtime api seam.

pub(crate) use super::frontier::plan_frontier_step;
pub(crate) use super::host::{
    build_pending_host_work_request_impl as build_pending_host_work_request,
    build_pending_host_work_requests_impl as build_pending_host_work_requests,
};
pub(crate) use super::instance::{
    BpmnInstanceState, EventCompetitionState, InstanceLifecycle, MultiInstanceCollectionKey,
    MultiInstanceCollectionKind, MultiInstanceCollectionSlot, MultiInstanceDataRuntimeState,
    MultiInstanceOutputCollectionState, NodeRuntimeStatus, SuspendReason,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state, complete_parallel_multi_instance_iteration, create_instance_impl,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    has_parallel_multi_instance_state, increment_sequential_multi_instance_iterations,
    increment_standard_loop_iterations, install_process_state,
    parallel_multi_instance_iteration_variables, parallel_multi_instance_min_token_id,
    parallel_multi_instance_state, parallel_multi_instance_state_mut, pop_call_activity_frame,
    push_call_activity_frame, register_parallel_multi_instance_iteration,
    resolve_process_for_instance, restore_call_activity_frame,
    sequential_multi_instance_iteration_variables, sequential_multi_instance_state,
    sequential_multi_instance_state_mut, standard_loop_completed_iterations,
};
pub(crate) use super::lifecycle::{
    advance_instance_impl, apply_pending_host_work_result_impl as apply_pending_host_work_result,
    push_active_token,
};
pub(crate) use super::wait::{
    apply_event_poll_outcome_impl as apply_event_poll_outcome,
    build_event_poll_request_impl as build_event_poll_request,
};
pub(crate) use crate::runtime_advance_api::BpmnAdvanceOutcome;
pub(crate) use crate::runtime_dispatch_api::{PendingHostWork, PendingHostWorkKind};
pub(crate) use crate::runtime_frontier_api::{
    BpmnFrontierExecutionBatch, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierParallelJoinMerge, BpmnFrontierPlanAction,
};
pub(crate) use crate::runtime_join_api::JoinRuntimeState;
pub(crate) use crate::runtime_token_api::TokenRecord;
pub(crate) use crate::runtime_wait_api::{WaitKind, WaitRegistration};
