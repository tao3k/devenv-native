use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep, BpmnFrontierPlanAction,
    BpmnInstanceInit, BpmnInstanceState, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    NodeRuntimeStatus, ProcessKey, TokenRecord, create_instance, merge_frontier_execution_steps,
    plan_frontier_step,
};
use serde_json::json;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

#[test]
#[ignore = "performance probe"]
fn performance_probe_runtime_frontier_planning_compares_public_snapshot_vs_direct_proposals() {
    let node_count = 20_000_u32;
    let token_count = 10_000_u64;
    let iterations = 128_u32;
    let process = frontier_probe_process("runtime_frontier_planning_probe", node_count);
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime_frontier_planning_probe",
        vec![process],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "runtime_frontier_planning_probe",
        BpmnInstanceInit::new("wf_runtime_frontier_planning_probe", json!({}), 1),
    )
    .must("runtime frontier planning probe instance should be created");
    instance.active_tokens = build_frontier_snapshot_probe_tokens(token_count, node_count);
    for node_state in &mut instance.node_states {
        node_state.status = NodeRuntimeStatus::Idle;
    }
    for token in &instance.active_tokens {
        instance.node_states[token.node_index as usize].status = NodeRuntimeStatus::Queued;
    }
    let process = &package.processes[0];

    let public_start = Instant::now();
    let mut public_proposal_sum = 0_usize;
    let mut public_step_sum = 0_usize;
    for _ in 0..iterations {
        let plan = plan_frontier_step(process, &instance);
        public_proposal_sum += frontier_action_proposal_weight(&plan.action);
        public_step_sum += frontier_action_step_weight(&plan.action);
    }
    let public_elapsed = public_start.elapsed();

    let direct_wrapped_start = Instant::now();
    let mut direct_wrapped_proposal_sum = 0_usize;
    let mut direct_wrapped_step_sum = 0_usize;
    for _ in 0..iterations {
        let proposals = direct_runtime_execution_proposals(&instance);
        let steps = merge_frontier_execution_steps(process, &proposals);
        direct_wrapped_proposal_sum += proposals.len();
        direct_wrapped_step_sum += frontier_step_weight(&steps);
    }
    let direct_wrapped_elapsed = direct_wrapped_start.elapsed();

    let runtime_fast_path_start = Instant::now();
    let mut runtime_fast_path_proposal_sum = 0_usize;
    for _ in 0..iterations {
        let proposals = direct_runtime_execution_proposals(&instance);
        runtime_fast_path_proposal_sum += proposals.len();
    }
    let runtime_fast_path_elapsed = runtime_fast_path_start.elapsed();

    assert_eq!(public_proposal_sum, direct_wrapped_proposal_sum);
    assert_eq!(public_proposal_sum, runtime_fast_path_proposal_sum);
    assert_eq!(public_step_sum, direct_wrapped_step_sum);
    assert_eq!(public_step_sum, public_proposal_sum);
    black_box((
        public_proposal_sum,
        public_step_sum,
        direct_wrapped_proposal_sum,
        direct_wrapped_step_sum,
        runtime_fast_path_proposal_sum,
    ));
    eprintln!(
        "performance_probe runtime_frontier_planning nodes={} tokens={} iterations={} public_snapshot_ms={:.3} direct_wrapped_steps_ms={:.3} runtime_fast_path_ms={:.3}",
        node_count,
        token_count,
        iterations,
        public_elapsed.as_secs_f64() * 1000.0,
        direct_wrapped_elapsed.as_secs_f64() * 1000.0,
        runtime_fast_path_elapsed.as_secs_f64() * 1000.0
    );
}

fn frontier_probe_process(process_id: &str, node_count: u32) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime_frontier_planning_probe",
            process_id,
            format!("digest_{process_id}"),
        ),
        linear_nodes(node_count),
        linear_edges(node_count),
        Vec::new(),
    )
}

fn linear_nodes(node_count: u32) -> Vec<BpmnNodeSpec> {
    (0..node_count)
        .map(|index| match index {
            0 => BpmnNodeSpec::new(index, format!("start_{index}"), BpmnNodeKind::StartEvent),
            i if i == node_count - 1 => {
                BpmnNodeSpec::new(index, format!("end_{index}"), BpmnNodeKind::EndEvent)
            }
            _ => BpmnNodeSpec::new(index, format!("task_{index}"), BpmnNodeKind::ServiceTask),
        })
        .collect()
}

fn linear_edges(node_count: u32) -> Vec<BpmnEdgeSpec> {
    (0..node_count - 1)
        .map(|index| BpmnEdgeSpec::new(index, index + 1, None::<&str>))
        .collect()
}

fn build_frontier_snapshot_probe_tokens(token_count: u64, node_count: u32) -> Vec<TokenRecord> {
    (0..token_count)
        .map(|offset| TokenRecord {
            token_id: offset + 1,
            node_index: 5 + u32::try_from(offset % u64::from(node_count - 5))
                .must("frontier snapshot probe token offset should fit in u32"),
            incoming_edge_index: Some(
                u32::try_from(offset % 8).must("frontier snapshot probe edge should fit in u32"),
            ),
            inclusive_join_hint: None,
        })
        .collect()
}

fn direct_runtime_execution_proposals(
    instance: &BpmnInstanceState,
) -> Vec<BpmnFrontierExecutionProposal> {
    instance
        .active_tokens
        .iter()
        .enumerate()
        .filter_map(|(token_index, token)| {
            let status = instance
                .node_states
                .get(token.node_index as usize)
                .map(|node_state| &node_state.status);
            (status == Some(&NodeRuntimeStatus::Queued)).then_some(BpmnFrontierExecutionProposal {
                token_id: token.token_id,
                token_index,
                node_index: token.node_index,
                incoming_edge_index: token.incoming_edge_index,
            })
        })
        .collect()
}

fn frontier_action_proposal_weight(action: &BpmnFrontierPlanAction) -> usize {
    match action {
        BpmnFrontierPlanAction::ExecuteBatch(batch) => batch.proposals.len(),
        BpmnFrontierPlanAction::BlockedOnHost(pending) => pending.len(),
        BpmnFrontierPlanAction::WaitingExternalEvent
        | BpmnFrontierPlanAction::Suspended(_)
        | BpmnFrontierPlanAction::Stalled => 0,
    }
}

fn frontier_action_step_weight(action: &BpmnFrontierPlanAction) -> usize {
    match action {
        BpmnFrontierPlanAction::ExecuteBatch(batch) => frontier_step_weight(&batch.steps),
        BpmnFrontierPlanAction::BlockedOnHost(_)
        | BpmnFrontierPlanAction::WaitingExternalEvent
        | BpmnFrontierPlanAction::Suspended(_)
        | BpmnFrontierPlanAction::Stalled => 0,
    }
}

fn frontier_step_weight(steps: &[BpmnFrontierExecutionStep]) -> usize {
    steps
        .iter()
        .map(|step| match step {
            BpmnFrontierExecutionStep::Proposal(_) => 1,
            BpmnFrontierExecutionStep::ParallelJoin(group) => group.proposals.len(),
        })
        .sum()
}
