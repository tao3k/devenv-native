use super::{
    BpmnFrontierExecutionBatch, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierParallelJoinMerge,
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

pub(crate) fn merge_execution_steps(
    process: &BpmnProcessSpec,
    proposals: &[BpmnFrontierExecutionProposal],
) -> Vec<BpmnFrontierExecutionStep> {
    let mut steps = Vec::new();
    let mut index = 0;
    while index < proposals.len() {
        let proposal = &proposals[index];
        if is_parallel_join_node(process, proposal.node_index) {
            let mut merged = vec![proposal.clone()];
            index += 1;
            while index < proposals.len()
                && proposals[index].node_index == proposal.node_index
                && is_parallel_join_node(process, proposals[index].node_index)
            {
                merged.push(proposals[index].clone());
                index += 1;
            }
            if merged.len() == 1 {
                if let Some(only_proposal) = merged.pop() {
                    steps.push(BpmnFrontierExecutionStep::Proposal(only_proposal));
                }
            } else {
                steps.push(BpmnFrontierExecutionStep::ParallelJoin(
                    BpmnFrontierParallelJoinMerge {
                        node_index: proposal.node_index,
                        proposals: merged,
                    },
                ));
            }
            continue;
        }

        steps.push(BpmnFrontierExecutionStep::Proposal(proposal.clone()));
        index += 1;
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
