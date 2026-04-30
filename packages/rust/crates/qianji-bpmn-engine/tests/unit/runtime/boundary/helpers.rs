use qianji_bpmn_engine::{
    BpmnEventKind, BpmnInstanceState, InstanceLifecycle, NodeRuntimeStatus, WaitKind,
};
use serde_json::json;

pub(super) fn assert_non_interrupting_boundary_branch_open(
    instance: &BpmnInstanceState,
    expected_key: &str,
) {
    assert_eq!(instance.pending_host_work.len(), 1);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens.len(), 2);
    assert!(
        instance
            .active_tokens
            .iter()
            .any(|token| token.node_index == 1)
    );
    assert!(
        instance
            .active_tokens
            .iter()
            .any(|token| token.node_index == 4)
    );
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Executing);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Queued);
    assert_eq!(instance.variables["amount"], json!(7));
    assert_eq!(instance.variables[expected_key], json!(true));
}

pub(super) fn assert_non_interrupting_boundary_branch_drained(instance: &BpmnInstanceState) {
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Completed);
}

pub(super) fn assert_non_interrupting_primary_path_resumed(
    instance: &BpmnInstanceState,
    expected_key: &str,
) {
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[3].status, NodeRuntimeStatus::Queued);
    assert_eq!(instance.variables["amount"], json!(7));
    assert_eq!(instance.variables[expected_key], json!(true));
    assert_eq!(instance.variables["approved"], json!(true));
}

pub(super) fn assert_non_interrupting_boundary_external_wait(
    instance: &BpmnInstanceState,
    expected_event_kind: BpmnEventKind,
    expected_reference: &str,
    expected_name: &str,
) {
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(expected_event_kind));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some(expected_reference)
    );
    assert_eq!(instance.waits[0].event_name.as_deref(), Some(expected_name));
    assert!(instance.waits[0].timer.is_none());
    assert_eq!(
        instance.waits[0].deduplication_key.as_deref(),
        Some(expected_reference)
    );
}

pub(super) fn assert_interrupting_boundary_external_wait(
    instance: &BpmnInstanceState,
    expected_event_kind: BpmnEventKind,
    expected_reference: &str,
    expected_name: &str,
) {
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(expected_event_kind));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some(expected_reference)
    );
    assert_eq!(instance.waits[0].event_name.as_deref(), Some(expected_name));
    assert!(instance.waits[0].timer.is_none());
    assert_eq!(
        instance.waits[0].deduplication_key.as_deref(),
        Some(expected_reference)
    );
}

pub(super) fn assert_interrupting_boundary_path_routed(
    instance: &BpmnInstanceState,
    expected_key: &str,
) {
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Cancelled);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Queued);
    assert_eq!(instance.variables["amount"], json!(7));
    assert_eq!(instance.variables[expected_key], json!(true));
}
