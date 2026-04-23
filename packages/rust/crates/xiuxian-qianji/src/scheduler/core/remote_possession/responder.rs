use crate::contracts::QianjiOutput;
use crate::error::QianjiError;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::types::REMOTE_POSSESSION_REQUEST_TTL_SECONDS;
use crate::scheduler_preflight::resolve_wendao_placeholders_in_context;
use crate::swarm::{RemoteNodeResponse, map_execution_error_to_response};
use tokio::time::Duration;

impl QianjiScheduler {
    /// Executes one node mechanism by node id for remote possession responders.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when the node id is unknown, context preflight fails,
    /// or the underlying mechanism returns an execution error.
    async fn execute_node_for_remote(
        &self,
        node_id: &str,
        context: serde_json::Value,
    ) -> Result<QianjiOutput, QianjiError> {
        let mechanism = {
            let engine = self.engine.read().await;
            let Some(index) = engine
                .graph
                .node_indices()
                .find(|idx| engine.graph[*idx].id == node_id)
            else {
                return Err(QianjiError::Execution(format!(
                    "remote possession target node not found: {node_id}"
                )));
            };
            engine.graph[index].mechanism.clone()
        };

        let preflight_context =
            resolve_wendao_placeholders_in_context(&context).map_err(QianjiError::Execution)?;
        mechanism
            .execute(&preflight_context)
            .await
            .map_err(QianjiError::Execution)
    }

    /// Processes one pending remote possession request for the given role class.
    ///
    /// Returns `Ok(true)` when one request was claimed and responded, `Ok(false)` when no
    /// request arrived during the timeout window.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when queue operations or response publication fail.
    pub(crate) async fn process_remote_possession_once(
        &self,
        role_class: &str,
        agent_id: &str,
        block_timeout: Duration,
    ) -> Result<bool, QianjiError> {
        let Some(bus) = &self.remote_possession_bus else {
            return Ok(false);
        };
        let request = bus
            .claim_next_for_role(role_class, agent_id, block_timeout)
            .await
            .map_err(|error| QianjiError::Execution(error.to_string()))?;
        let Some(request) = request else {
            return Ok(false);
        };

        let response = match self
            .execute_node_for_remote(request.node_id.as_str(), request.context.clone())
            .await
        {
            Ok(output) => {
                RemoteNodeResponse::success(&request, self.cluster_id.as_str(), agent_id, output)
            }
            Err(error) => map_execution_error_to_response(
                &request,
                self.cluster_id.as_str(),
                agent_id,
                error.to_string().as_str(),
            ),
        };

        bus.submit_response(&response, REMOTE_POSSESSION_REQUEST_TTL_SECONDS)
            .await
            .map_err(|error| QianjiError::Execution(error.to_string()))?;
        Ok(true)
    }
}
