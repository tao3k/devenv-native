use crate::error::QianjiError;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::state::NodeExecutionResult;
use futures::{StreamExt, stream::FuturesUnordered};
use petgraph::stable_graph::NodeIndex;

impl QianjiScheduler {
    /// Execute the graph following Synaptic-Flow: entropy-aware dependency resolution.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when delegated checkpoint execution fails,
    /// including mechanism failures, task panics, or drift safeguards.
    pub async fn run(
        &self,
        initial_context: serde_json::Value,
    ) -> Result<serde_json::Value, QianjiError> {
        self.run_with_checkpoint(initial_context, None, None).await
    }

    /// Execute the graph with state persistence and resumption via Valkey (Redis).
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when checkpoint load/save/delete fails,
    /// when any mechanism aborts/errors, on task panic, or when step budget is exceeded.
    pub async fn run_with_checkpoint(
        &self,
        initial_context: serde_json::Value,
        session_id: Option<String>,
        redis_url: Option<String>,
    ) -> Result<serde_json::Value, QianjiError> {
        let (mut context, mut active_branches, mut total_steps) = self
            .load_checkpoint_state(
                &initial_context,
                session_id.as_deref(),
                redis_url.as_deref(),
            )
            .await;

        let mut exec_state = self.rebuild_exec_state(&active_branches).await;
        let mut executing_tasks: FuturesUnordered<
            tokio::task::JoinHandle<(NodeIndex, NodeExecutionResult)>,
        > = FuturesUnordered::new();

        loop {
            if total_steps > self.max_total_steps {
                return Err(QianjiError::Drift(
                    "Maximum execution steps exceeded (Potential infinite loop)".to_string(),
                ));
            }

            let deferred_nodes = self
                .launch_ready_nodes(
                    &mut exec_state,
                    &context,
                    session_id.as_deref(),
                    &mut executing_tasks,
                )
                .await;

            if executing_tasks.is_empty() && deferred_nodes.is_empty() {
                break;
            }

            if executing_tasks.is_empty() {
                if let Some(suspended_context) = self
                    .process_deferred_nodes(
                        &deferred_nodes,
                        &mut context,
                        &mut active_branches,
                        &mut total_steps,
                        session_id.as_deref(),
                        redis_url.as_deref(),
                    )
                    .await?
                {
                    return Ok(suspended_context);
                }
                exec_state = self.rebuild_exec_state(&active_branches).await;
                continue;
            }

            if let Some(join_result) = executing_tasks.next().await {
                total_steps += 1;
                if let Some(suspended_context) = self
                    .process_completed_task(
                        join_result,
                        &mut context,
                        &mut active_branches,
                        total_steps,
                        session_id.as_deref(),
                        redis_url.as_deref(),
                    )
                    .await?
                {
                    return Ok(suspended_context);
                }
                exec_state = self.rebuild_exec_state(&active_branches).await;
            }
        }

        if !self.execution_identity.is_configured() {
            self.delete_checkpoint_if_needed(session_id.as_deref(), redis_url.as_deref())
                .await;
        }

        Ok(context)
    }
}
