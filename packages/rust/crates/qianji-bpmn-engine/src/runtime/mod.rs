//! Runtime state shells for BPMN workflow instances.

mod dispatch;
mod frontier;
mod host;
mod instance;
mod join;
mod lifecycle;
mod token;
mod wait;

pub use dispatch::{PendingHostWork, PendingHostWorkKind};
pub use frontier::{
    BpmnFrontierEntry, BpmnFrontierEntryStatus, BpmnFrontierExecutionBatch,
    BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep, BpmnFrontierParallelJoinMerge,
    BpmnFrontierPlan, BpmnFrontierPlanAction, BpmnFrontierProposalSet, BpmnFrontierSnapshot,
    collect_frontier_proposals, merge_frontier_execution_steps, plan_frontier_step,
    reduce_frontier_plan, snapshot_frontier,
};
pub use host::{build_pending_host_work_request, build_pending_host_work_requests};
pub use instance::{
    BpmnInstanceInit, BpmnInstanceState, CallActivityFrame, EventCompetitionState,
    InstanceLifecycle, NodeRuntimeState, NodeRuntimeStatus, SequentialMultiInstanceState,
    StandardLoopState, SuspendReason, create_instance,
};
pub(crate) use instance::{
    clear_sequential_multi_instance_state, clear_standard_loop_state,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    increment_sequential_multi_instance_iterations, increment_standard_loop_iterations,
    install_process_state, pop_call_activity_frame, push_call_activity_frame,
    resolve_process_for_instance, restore_call_activity_frame, sequential_multi_instance_progress,
    standard_loop_completed_iterations,
};
pub use join::JoinRuntimeState;
pub use lifecycle::{BpmnAdvanceOutcome, advance_instance, apply_pending_host_work_result};
pub use token::TokenRecord;
pub use wait::{WaitKind, WaitRegistration, apply_event_poll_outcome, build_event_poll_request};
