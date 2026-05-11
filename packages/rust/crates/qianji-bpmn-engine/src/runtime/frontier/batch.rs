use super::{
    BpmnFrontierExecutionBatch, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierParallelJoinMerge, BpmnFrontierRuntimeBatch,
};
use crate::ir::BpmnProcessSpec;
use crate::ir_index_api::BpmnNodeIndex;
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};

fn is_parallel_join_node(process: &BpmnProcessSpec, node_index: BpmnNodeIndex) -> bool {
    let node = &process.nodes[node_index as usize];
    node.kind == BpmnNodeKind::Gateway
        && node.gateway_kind == Some(BpmnGatewayKind::Parallel)
        && process.incoming_edge_indices(node_index).len() > 1
}

fn push_proposal_run(
    steps: &mut Vec<BpmnFrontierExecutionStep>,
    node_index: BpmnNodeIndex,
    mut proposals: Vec<BpmnFrontierExecutionProposal>,
) {
    if proposals.len() == 1 {
        if let Some(only_proposal) = proposals.pop() {
            steps.push(BpmnFrontierExecutionStep::Proposal(only_proposal));
        }
    } else {
        steps.push(BpmnFrontierExecutionStep::ParallelJoin(
            BpmnFrontierParallelJoinMerge {
                node_index,
                proposals,
            },
        ));
    }
}

fn collect_parallel_join_run(
    process: &BpmnProcessSpec,
    proposals: &[BpmnFrontierExecutionProposal],
    index: &mut usize,
) -> Vec<BpmnFrontierExecutionProposal> {
    let node_index = proposals[*index].node_index;
    let mut merged = Vec::new();
    while *index < proposals.len()
        && proposals[*index].node_index == node_index
        && is_parallel_join_node(process, proposals[*index].node_index)
    {
        merged.push(proposals[*index].clone());
        *index += 1;
    }
    merged
}

pub(crate) fn merge_execution_steps(
    process: &BpmnProcessSpec,
    proposals: &[BpmnFrontierExecutionProposal],
) -> Vec<BpmnFrontierExecutionStep> {
    let mut steps = Vec::new();
    let mut index = 0;
    while index < proposals.len() {
        let proposal = &proposals[index];
        if is_parallel_join_node(process, proposal.node_index) {
            let node_index = proposal.node_index;
            let merged = collect_parallel_join_run(process, proposals, &mut index);
            push_proposal_run(&mut steps, node_index, merged);
            continue;
        }

        steps.push(BpmnFrontierExecutionStep::Proposal(proposal.clone()));
        index += 1;
    }
    steps
}

fn collect_parallel_join_run_owned(
    node_index: BpmnNodeIndex,
    proposals: &mut std::iter::Peekable<std::vec::IntoIter<BpmnFrontierExecutionProposal>>,
    first: BpmnFrontierExecutionProposal,
) -> Vec<BpmnFrontierExecutionProposal> {
    let mut merged = vec![first];
    while proposals
        .peek()
        .is_some_and(|next| next.node_index == node_index)
    {
        if let Some(next) = proposals.next() {
            merged.push(next);
        }
    }
    merged
}

pub(crate) fn merge_execution_steps_owned(
    process: &BpmnProcessSpec,
    proposals: Vec<BpmnFrontierExecutionProposal>,
) -> Vec<BpmnFrontierExecutionStep> {
    let mut steps = Vec::new();
    let mut proposals = proposals.into_iter().peekable();
    while let Some(proposal) = proposals.next() {
        if is_parallel_join_node(process, proposal.node_index) {
            let node_index = proposal.node_index;
            let merged = collect_parallel_join_run_owned(node_index, &mut proposals, proposal);
            push_proposal_run(&mut steps, node_index, merged);
            continue;
        }

        steps.push(BpmnFrontierExecutionStep::Proposal(proposal));
    }
    steps
}

pub(crate) fn build_frontier_execution_batch(
    process: &BpmnProcessSpec,
    proposals: Vec<BpmnFrontierExecutionProposal>,
) -> BpmnFrontierExecutionBatch {
    let steps = merge_execution_steps(process, &proposals);
    BpmnFrontierExecutionBatch { proposals, steps }
}

pub(crate) fn build_frontier_runtime_batch(
    process: &BpmnProcessSpec,
    proposals: Vec<BpmnFrontierExecutionProposal>,
) -> BpmnFrontierRuntimeBatch {
    if proposals
        .iter()
        .any(|proposal| is_parallel_join_node(process, proposal.node_index))
    {
        return BpmnFrontierRuntimeBatch::Steps(merge_execution_steps_owned(process, proposals));
    }
    BpmnFrontierRuntimeBatch::Proposals(proposals)
}
