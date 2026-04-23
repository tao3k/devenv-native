use crate::error::QianjiError;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::types::{EXTERNAL_PROGRESS_TIMEOUT_MS, EXTERNAL_PROGRESS_WAIT_MS};
use crate::scheduler_checkpoint::QianjiStateSnapshot;
use crate::scheduler_state::merge_output_data;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;
use tokio::time::{Duration, Instant, sleep};

impl QianjiScheduler {
    pub(in crate::scheduler::core) async fn wait_for_external_progress(
        &self,
        deferred_nodes: &[NodeIndex],
        context: &mut serde_json::Value,
        active_branches: &mut HashSet<String>,
        total_steps: &mut u32,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) -> Result<bool, QianjiError> {
        let node_ids = self.node_ids_for_indices(deferred_nodes).await;
        let (Some(sid), Some(url)) = (session_id, redis_url) else {
            return Err(QianjiError::Execution(format!(
                "node ownership filtered runnable nodes {node_ids:?}, but checkpoint is disabled"
            )));
        };

        let deadline = Instant::now() + Duration::from_millis(EXTERNAL_PROGRESS_TIMEOUT_MS);
        while Instant::now() < deadline {
            match QianjiStateSnapshot::load(sid, url).await {
                Ok(Some(snapshot)) => {
                    let status_changed = self.apply_snapshot_node_statuses(&snapshot).await;
                    let previous_steps = *total_steps;
                    if snapshot.total_steps > *total_steps {
                        *total_steps = snapshot.total_steps;
                    }

                    let mut merged_context = context.clone();
                    merge_output_data(&mut merged_context, &snapshot.context);
                    let context_changed = *context != merged_context;
                    if context_changed {
                        *context = merged_context;
                    }

                    let branches_changed = *active_branches != snapshot.active_branches;
                    if branches_changed {
                        active_branches.clone_from(&snapshot.active_branches);
                    }

                    if status_changed
                        || context_changed
                        || branches_changed
                        || *total_steps != previous_steps
                    {
                        return Ok(true);
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    log::warn!(
                        "Failed to poll external checkpoint progress for session {sid}: {error}"
                    );
                }
            }
            sleep(Duration::from_millis(EXTERNAL_PROGRESS_WAIT_MS)).await;
        }

        log::warn!("Timed out waiting for externally-owned nodes to progress: {node_ids:?}");
        Ok(false)
    }
}
