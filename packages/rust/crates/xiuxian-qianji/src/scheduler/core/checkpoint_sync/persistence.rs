use crate::scheduler::core::QianjiScheduler;
use crate::scheduler_checkpoint::QianjiStateSnapshot;
use crate::scheduler_state::merge_output_data;
use std::collections::{HashMap, HashSet};

impl QianjiScheduler {
    pub(in crate::scheduler) async fn load_checkpoint_state(
        &self,
        initial_context: &serde_json::Value,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) -> (serde_json::Value, HashSet<String>, u32) {
        let mut context = initial_context.clone();
        let mut active_branches: HashSet<String> = HashSet::new();
        let mut total_steps = 0;

        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return (context, active_branches, total_steps);
        };

        match QianjiStateSnapshot::load(sid, url).await {
            Ok(Some(snapshot)) => {
                let mut merged_context = snapshot.context.clone();
                merge_output_data(&mut merged_context, initial_context);
                context = merged_context;
                active_branches = snapshot.active_branches.clone();
                total_steps = snapshot.total_steps;
                let _ = self.apply_snapshot_node_statuses(&snapshot).await;
            }
            Ok(None) => {}
            Err(error) => {
                log::warn!("Failed to load checkpoint for session {sid}: {error}. Starting fresh.");
            }
        }

        (context, active_branches, total_steps)
    }

    pub(in crate::scheduler::core) async fn save_checkpoint_if_needed(
        &self,
        session_id: Option<&str>,
        redis_url: Option<&str>,
        total_steps: u32,
        active_branches: &HashSet<String>,
        context: &serde_json::Value,
    ) {
        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return;
        };

        let engine = self.engine.read().await;
        let mut node_statuses = HashMap::new();
        for node_idx in engine.graph.node_indices() {
            let node = &engine.graph[node_idx];
            node_statuses.insert(node.id.clone(), node.status.clone());
        }

        let snapshot = QianjiStateSnapshot {
            session_id: sid.to_string(),
            total_steps,
            active_branches: active_branches.clone(),
            context: context.clone(),
            node_statuses,
        };

        if let Err(error) = snapshot.save(url).await {
            log::warn!("Failed to save checkpoint for session {sid}: {error}");
        }
    }

    pub(in crate::scheduler) async fn delete_checkpoint_if_needed(
        &self,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) {
        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return;
        };
        let _ = QianjiStateSnapshot::delete(sid, url).await;
    }
}
