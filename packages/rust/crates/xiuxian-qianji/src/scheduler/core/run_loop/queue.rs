use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::state::{ExecutionState, NodeExecutionResult, spawn_node_execution_task};
use crate::telemetry::NodeTransitionPhase;
use futures::stream::FuturesUnordered;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;

impl QianjiScheduler {
    pub(super) async fn rebuild_exec_state(
        &self,
        active_branches: &HashSet<String>,
    ) -> ExecutionState {
        let engine = self.engine.read().await;
        ExecutionState::build(&engine, active_branches)
    }

    pub(super) async fn launch_ready_nodes(
        &self,
        exec_state: &mut ExecutionState,
        context: &serde_json::Value,
        session_id: Option<&str>,
        executing_tasks: &mut FuturesUnordered<
            tokio::task::JoinHandle<(NodeIndex, NodeExecutionResult)>,
        >,
    ) -> Vec<NodeIndex> {
        let mut deferred_nodes = Vec::new();
        while let Some(node_idx) = exec_state.ready_queue.pop_front() {
            if self.should_execute_node(node_idx).await {
                self.emit_node_transition(node_idx, NodeTransitionPhase::Entering, session_id)
                    .await;
                executing_tasks.push(spawn_node_execution_task(
                    self.engine.clone(),
                    node_idx,
                    context.clone(),
                ));
            } else {
                deferred_nodes.push(node_idx);
            }
        }
        deferred_nodes
    }
}
