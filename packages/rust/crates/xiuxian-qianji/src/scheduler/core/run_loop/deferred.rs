use crate::error::QianjiError;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::types::RemoteDelegationOutcome;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;

impl QianjiScheduler {
    pub(in crate::scheduler) async fn process_deferred_nodes(
        &self,
        deferred_nodes: &[NodeIndex],
        context: &mut serde_json::Value,
        active_branches: &mut HashSet<String>,
        total_steps: &mut u32,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) -> Result<Option<serde_json::Value>, QianjiError> {
        match self
            .attempt_remote_possession(deferred_nodes, context, active_branches, session_id)
            .await?
        {
            RemoteDelegationOutcome::Suspend(suspended_context) => {
                return Ok(Some(suspended_context));
            }
            RemoteDelegationOutcome::Progressed => return Ok(None),
            RemoteDelegationOutcome::Noop => {}
        }

        let progressed = self
            .wait_for_external_progress(
                deferred_nodes,
                context,
                active_branches,
                total_steps,
                session_id,
                redis_url,
            )
            .await?;
        if !progressed {
            return Err(QianjiError::Execution(
                "No external progress observed for deferred swarm nodes".to_string(),
            ));
        }
        Ok(None)
    }
}
