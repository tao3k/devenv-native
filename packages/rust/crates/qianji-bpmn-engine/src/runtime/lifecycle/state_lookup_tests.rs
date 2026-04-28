use super::*;
use crate::ir::ProcessKey;
use crate::runtime::lifecycle::scope::{InstanceLifecycle, TokenRecord};
use crate::runtime_instance_api::BpmnInstanceState;
use std::sync::Arc;

#[test]
fn shifted_frontier_index_resolves_after_prefix_token_removal_without_hashmap() {
    let mut instance = empty_instance();
    instance.active_tokens = vec![token(99, 1), token(10, 2), token(11, 2), token(12, 2)];
    let mut lookup = FrontierTokenLookup::default();
    let first = proposal(10, 1, 2);
    let second = proposal(11, 2, 2);

    let Some(first_index) = lookup.resolve_frontier_proposal_token_index(&instance, &first) else {
        panic!("first proposal should resolve by snapshot index");
    };
    assert_eq!(first_index, 1);

    instance.active_tokens.remove(first_index);
    lookup.observe_frontier_proposal_execution(&instance, &first, first_index);

    let Some(second_index) = lookup.resolve_frontier_proposal_token_index(&instance, &second)
    else {
        panic!("second proposal should resolve by shifted snapshot index");
    };
    assert_eq!(second_index, 1);
    assert!(lookup.token_indices.is_none());
    assert_eq!(lookup.queries_since_refresh, 0);
    assert_eq!(lookup.frontier_index_shift, 1);
}

#[test]
fn fallback_lookup_still_handles_non_prefix_token_removal() {
    let mut instance = empty_instance();
    instance.active_tokens = vec![token(10, 2), token(11, 2), token(12, 2)];
    let mut lookup = FrontierTokenLookup::default();
    let first = proposal(10, 0, 2);
    let third = proposal(12, 2, 2);

    let Some(first_index) = lookup.resolve_frontier_proposal_token_index(&instance, &first) else {
        panic!("first proposal should resolve by snapshot index");
    };
    instance.active_tokens.remove(first_index);
    instance.active_tokens.remove(0);
    lookup.observe_frontier_proposal_execution(&instance, &first, first_index);

    let Some(third_index) = lookup.resolve_frontier_proposal_token_index(&instance, &third) else {
        panic!("third proposal should fall back to token id after extra removal");
    };
    assert_eq!(third_index, 0);
    assert!(lookup.token_indices.is_none());
    assert_eq!(lookup.queries_since_refresh, 1);
}

fn empty_instance() -> BpmnInstanceState {
    BpmnInstanceState {
        instance_id: Arc::from("wf_frontier_lookup"),
        process: ProcessKey::new("pkg_frontier_lookup", "process_frontier_lookup", "digest"),
        process_index: 0,
        call_stack: Vec::new(),
        sequence: 0,
        next_token_id: 0,
        lifecycle: InstanceLifecycle::Running,
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
        human_task_events: Vec::new(),
        suspend_reason: None,
        updated_at_ms: 0,
    }
}

fn token(token_id: u64, node_index: u32) -> TokenRecord {
    TokenRecord {
        token_id,
        node_index,
        incoming_edge_index: Some(0),
        inclusive_join_hint: None,
    }
}

fn proposal(token_id: u64, token_index: usize, node_index: u32) -> BpmnFrontierExecutionProposal {
    BpmnFrontierExecutionProposal {
        token_id,
        token_index,
        node_index,
        incoming_edge_index: Some(0),
    }
}
