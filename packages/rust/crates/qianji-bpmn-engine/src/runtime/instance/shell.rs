use crate::error::{BpmnEngineError, Result};
use crate::ir::{BpmnPackage, BpmnProcessSpec};
pub(crate) use crate::runtime_instance_api::{
    BpmnInstanceInit, BpmnInstanceState, CallActivityFrame, EventCompetitionState,
    InstanceLifecycle, NodeRuntimeState, NodeRuntimeStatus, SuspendReason,
};
use std::borrow::Borrow;

pub(crate) fn build_node_states(process: &BpmnProcessSpec) -> Vec<NodeRuntimeState> {
    process
        .nodes
        .iter()
        .map(|node| NodeRuntimeState {
            node_index: node.index,
            status: NodeRuntimeStatus::Idle,
        })
        .collect()
}

pub(crate) fn create_instance_impl(
    package: impl Borrow<BpmnPackage>,
    process_id: &str,
    init: BpmnInstanceInit,
) -> Result<BpmnInstanceState> {
    let package = package.borrow();
    let (process_index, process) = package.find_process_position(process_id).ok_or_else(|| {
        BpmnEngineError::MissingProcess {
            process_id: (process_id.to_string()).into(),
        }
    })?;
    Ok(BpmnInstanceState {
        instance_id: (init.instance_id),
        process: process.key.clone(),
        process_index,
        call_stack: Vec::new(),
        sequence: 0,
        next_token_id: 1,
        lifecycle: InstanceLifecycle::Ready,
        variables: init.initial_variables,
        node_states: build_node_states(process),
        active_tokens: Vec::new(),
        trace: Vec::new(),
        joins: Vec::new(),
        standard_loops: Vec::new(),
        sequential_multi_instances: Vec::new(),
        parallel_multi_instances: Vec::new(),
        waits: Vec::new(),
        event_competition: None,
        detached_transaction_compensation: None,
        pending_host_work: Vec::new(),
        human_task_events: Vec::new(),
        suspend_reason: None,
        updated_at_ms: init.initial_timestamp_ms,
    })
}
