use super::*;
use crate::ir::ProcessKey;
use crate::runtime::lifecycle::scope::{InstanceLifecycle, PendingHostWorkKind};
use crate::runtime::lifecycle::state::push_active_token_with_allocator;
use crate::runtime_repeat_api::{ParallelMultiInstanceIterationState, ParallelMultiInstanceState};
use std::sync::Arc;

#[test]
fn allocation_uses_persisted_cursor_after_prior_tokens_disappear() {
    let mut instance = empty_instance();
    instance.sequence = 3;
    instance.next_token_id = 91;
    instance.active_tokens = vec![token(7)];

    assert_eq!(allocate_token_id(&mut instance), 91);
    assert_eq!(instance.next_token_id, 92);
}

#[test]
fn allocation_recovers_cursor_from_legacy_checkpoint_token_state() {
    let mut instance = empty_instance();
    instance.sequence = 5;
    instance.active_tokens = vec![token(17)];
    instance.pending_host_work = vec![pending_host_work(23)];
    instance.parallel_multi_instances = vec![parallel_multi_instance(29)];
    instance.call_stack = vec![call_frame(
        vec![token(31)],
        vec![pending_host_work(37)],
        vec![parallel_multi_instance(41)],
    )];

    assert_eq!(allocate_token_id(&mut instance), 42);
    assert_eq!(instance.next_token_id, 43);
}

#[test]
fn batch_allocator_reserves_every_issued_token_on_the_instance() {
    let mut instance = empty_instance();
    instance.next_token_id = 10;
    let mut allocator = token_id_allocator(&instance);

    assert_eq!(
        push_active_token_with_allocator(&mut instance, 1, 2, &mut allocator),
        10
    );
    assert_eq!(
        push_active_token_with_allocator(&mut instance, 2, 3, &mut allocator),
        11
    );
    assert_eq!(instance.next_token_id, 12);
    assert_eq!(
        instance
            .active_tokens
            .iter()
            .map(|token| token.token_id)
            .collect::<Vec<_>>(),
        vec![10, 11]
    );
}

fn empty_instance() -> BpmnInstanceState {
    BpmnInstanceState {
        instance_id: Arc::from("wf_token_cursor"),
        process: process_key(),
        process_index: 0,
        call_stack: Vec::new(),
        sequence: 0,
        next_token_id: 0,
        lifecycle: InstanceLifecycle::Ready,
        variables: serde_json::json!({}),
        node_states: Vec::new(),
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
        suspend_reason: None,
        updated_at_ms: 0,
    }
}

fn process_key() -> ProcessKey {
    ProcessKey::new("pkg_token_cursor", "process_token_cursor", "digest")
}

fn token(token_id: u64) -> TokenRecord {
    TokenRecord {
        token_id,
        node_index: 0,
        incoming_edge_index: None,
        inclusive_join_hint: None,
    }
}

fn pending_host_work(token_id: u64) -> PendingHostWork {
    PendingHostWork {
        token_id,
        process_id: Some("process_token_cursor".to_string()),
        node_index: 0,
        activity_id: Some("Task_0".to_string()),
        kind: PendingHostWorkKind::Service,
        decision: None,
        script_format: None,
        script_body: None,
        human_task_form: None,
        human_task_assignment: None,
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: None,
    }
}

fn parallel_multi_instance(token_id: u64) -> ParallelMultiInstanceState {
    ParallelMultiInstanceState {
        node_index: 0,
        total_iterations: 1,
        completed_iterations: 0,
        data_binding: None,
        active_iterations: vec![ParallelMultiInstanceIterationState {
            token_id,
            iteration_index: 0,
        }],
    }
}

fn call_frame(
    active_tokens: Vec<TokenRecord>,
    pending_host_work: Vec<PendingHostWork>,
    parallel_multi_instances: Vec<ParallelMultiInstanceState>,
) -> CallActivityFrame {
    CallActivityFrame {
        process: process_key(),
        process_index: 0,
        return_node_index: 0,
        node_states: Vec::new(),
        active_tokens,
        joins: Vec::new(),
        standard_loops: Vec::new(),
        sequential_multi_instances: Vec::new(),
        parallel_multi_instances,
        waits: Vec::new(),
        event_competition: None,
        pending_host_work,
        suspend_reason: None,
        transaction_cancel_variables: None,
        transaction_compensation: None,
    }
}
