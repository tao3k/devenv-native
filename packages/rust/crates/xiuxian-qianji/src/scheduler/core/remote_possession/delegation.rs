use crate::contracts::{NodeStatus, QianjiOutput};
use crate::error::QianjiError;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::types::{
    REMOTE_POSSESSION_MAX_WAIT_MS, REMOTE_POSSESSION_REQUEST_TTL_SECONDS, RemoteDelegationOutcome,
};
use crate::scheduler_state::merge_output_data;
use crate::swarm::RemoteNodeRequest;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;
use tokio::time::Duration;

impl QianjiScheduler {
    pub(super) async fn apply_remote_node_output(
        &self,
        node_idx: NodeIndex,
        output: QianjiOutput,
        context: &mut serde_json::Value,
        active_branches: &mut HashSet<String>,
    ) -> Result<Option<serde_json::Value>, QianjiError> {
        merge_output_data(context, &output.data);
        self.set_node_status(node_idx, NodeStatus::Completed).await;
        let suspend_reason = self
            .apply_instruction(output.instruction, active_branches)
            .await?;
        if let Some(reason) = suspend_reason {
            log::info!("Workflow suspended: {reason}");
            return Ok(Some(context.clone()));
        }
        Ok(None)
    }

    pub(in crate::scheduler::core) async fn attempt_remote_possession(
        &self,
        deferred_nodes: &[NodeIndex],
        context: &mut serde_json::Value,
        active_branches: &mut HashSet<String>,
        session_id: Option<&str>,
    ) -> Result<RemoteDelegationOutcome, QianjiError> {
        let (Some(bus), Some(sid)) = (&self.remote_possession_bus, session_id) else {
            return Ok(RemoteDelegationOutcome::Noop);
        };
        let requester_agent_id = self
            .execution_identity
            .agent_id
            .as_deref()
            .unwrap_or("unknown_agent");

        for node_idx in deferred_nodes {
            let Some(role_class) = self.deferred_node_role(*node_idx).await else {
                continue;
            };
            let Some(node_id) = self.deferred_node_id(*node_idx).await else {
                continue;
            };
            let request = RemoteNodeRequest::new(
                sid,
                node_id,
                role_class,
                self.cluster_id.clone(),
                requester_agent_id.to_string(),
                context.clone(),
            );
            let response = bus
                .request_and_wait(
                    &request,
                    REMOTE_POSSESSION_REQUEST_TTL_SECONDS,
                    Duration::from_millis(REMOTE_POSSESSION_MAX_WAIT_MS),
                )
                .await
                .map_err(|error| QianjiError::Execution(error.to_string()))?;
            let Some(response) = response else {
                continue;
            };
            if !response.ok {
                let message = response
                    .error
                    .unwrap_or_else(|| "remote possession execution failed".to_string());
                return Err(QianjiError::Execution(message));
            }
            let output = response.output.ok_or_else(|| {
                QianjiError::Execution("remote possession returned empty output".to_string())
            })?;
            if let Some(suspended_context) = self
                .apply_remote_node_output(*node_idx, output, context, active_branches)
                .await?
            {
                return Ok(RemoteDelegationOutcome::Suspend(suspended_context));
            }
            return Ok(RemoteDelegationOutcome::Progressed);
        }
        Ok(RemoteDelegationOutcome::Noop)
    }
}
