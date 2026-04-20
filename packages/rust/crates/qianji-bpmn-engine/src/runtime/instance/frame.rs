use super::process::build_node_states;
use super::shell::{BpmnInstanceState, CallActivityFrame};
use crate::ir::BpmnProcessSpec;
use crate::ir_index_api::BpmnNodeIndex;

pub(crate) fn push_call_activity_frame(
    instance: &mut BpmnInstanceState,
    return_node_index: BpmnNodeIndex,
    transaction_cancel_variables: Option<serde_json::Value>,
) {
    instance.call_stack.push(CallActivityFrame {
        process: instance.process.clone(),
        process_index: instance.process_index,
        return_node_index,
        node_states: std::mem::take(&mut instance.node_states),
        active_tokens: std::mem::take(&mut instance.active_tokens),
        joins: std::mem::take(&mut instance.joins),
        standard_loops: std::mem::take(&mut instance.standard_loops),
        sequential_multi_instances: std::mem::take(&mut instance.sequential_multi_instances),
        parallel_multi_instances: std::mem::take(&mut instance.parallel_multi_instances),
        waits: std::mem::take(&mut instance.waits),
        event_competition: instance.event_competition.take(),
        pending_host_work: std::mem::take(&mut instance.pending_host_work),
        suspend_reason: instance.suspend_reason.take(),
        transaction_cancel_variables,
    });
}

pub(crate) fn pop_call_activity_frame(
    instance: &mut BpmnInstanceState,
) -> Option<CallActivityFrame> {
    instance.call_stack.pop()
}

pub(crate) fn install_process_state(
    instance: &mut BpmnInstanceState,
    process: &BpmnProcessSpec,
    process_index: u32,
) {
    instance.process = process.key.clone();
    instance.process_index = process_index;
    instance.node_states = build_node_states(process);
    instance.active_tokens.clear();
    instance.joins.clear();
    instance.standard_loops.clear();
    instance.sequential_multi_instances.clear();
    instance.parallel_multi_instances.clear();
    instance.waits.clear();
    instance.event_competition = None;
    instance.pending_host_work.clear();
    instance.suspend_reason = None;
}

pub(crate) fn restore_call_activity_frame(
    instance: &mut BpmnInstanceState,
    frame: CallActivityFrame,
) -> BpmnNodeIndex {
    let return_node_index = frame.return_node_index;
    instance.process = frame.process;
    instance.process_index = frame.process_index;
    instance.node_states = frame.node_states;
    instance.active_tokens = frame.active_tokens;
    instance.joins = frame.joins;
    instance.standard_loops = frame.standard_loops;
    instance.sequential_multi_instances = frame.sequential_multi_instances;
    instance.parallel_multi_instances = frame.parallel_multi_instances;
    instance.waits = frame.waits;
    instance.event_competition = frame.event_competition;
    instance.pending_host_work = frame.pending_host_work;
    instance.suspend_reason = frame.suspend_reason;
    return_node_index
}
