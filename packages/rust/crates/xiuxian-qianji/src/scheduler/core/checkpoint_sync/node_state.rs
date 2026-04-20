use crate::contracts::NodeStatus;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler_checkpoint::QianjiStateSnapshot;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;

impl QianjiScheduler {
    pub(in crate::scheduler::core) async fn reset_retry_nodes(&self, node_ids: &[String]) {
        let mut engine = self.engine.write().await;
        let mut to_reset = HashSet::new();

        let initial_indices: Vec<_> = engine
            .graph
            .node_indices()
            .filter(|&idx| node_ids.contains(&engine.graph[idx].id))
            .collect();

        for start_idx in initial_indices {
            let mut bfs = petgraph::visit::Bfs::new(&engine.graph, start_idx);
            while let Some(visited) = bfs.next(&engine.graph) {
                to_reset.insert(visited);
            }
        }

        for idx in to_reset {
            engine.graph[idx].status = NodeStatus::Idle;
        }
    }

    pub(in crate::scheduler::core) async fn apply_snapshot_node_statuses(
        &self,
        snapshot: &QianjiStateSnapshot,
    ) -> bool {
        let mut changed = false;
        let mut engine = self.engine.write().await;
        let indices: Vec<_> = engine.graph.node_indices().collect();
        for node_idx in indices {
            let id = engine.graph[node_idx].id.clone();
            if let Some(status) = snapshot.node_statuses.get(&id)
                && engine.graph[node_idx].status != *status
            {
                engine.graph[node_idx].status = status.clone();
                changed = true;
            }
        }
        changed
    }

    pub(in crate::scheduler::core) async fn set_node_status(
        &self,
        node_idx: NodeIndex,
        status: NodeStatus,
    ) {
        let mut engine = self.engine.write().await;
        engine.graph[node_idx].status = status;
    }

    pub(in crate::scheduler::core) async fn node_ids_for_indices(
        &self,
        nodes: &[NodeIndex],
    ) -> Vec<String> {
        let engine = self.engine.read().await;
        nodes
            .iter()
            .filter_map(|index| engine.graph.node_weight(*index).map(|node| node.id.clone()))
            .collect()
    }
}
