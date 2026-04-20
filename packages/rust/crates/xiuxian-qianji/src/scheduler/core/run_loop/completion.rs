use crate::contracts::NodeStatus;
use crate::error::QianjiError;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler::types::{ConsensusCheckpointView, ConsensusOutcome};
use crate::scheduler_state::{NodeExecutionResult, merge_output_data};
use crate::telemetry::NodeTransitionPhase;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;

impl QianjiScheduler {
    pub(in crate::scheduler) async fn process_completed_task(
        &self,
        join_result: Result<(NodeIndex, NodeExecutionResult), tokio::task::JoinError>,
        context: &mut serde_json::Value,
        active_branches: &mut HashSet<String>,
        total_steps: u32,
        session_id: Option<&str>,
        redis_url: Option<&str>,
    ) -> Result<Option<serde_json::Value>, QianjiError> {
        match join_result {
            Ok((node_idx, Ok(output))) => {
                let checkpoint = ConsensusCheckpointView {
                    session_id,
                    redis_url,
                    total_steps,
                    active_branches,
                    context,
                };
                let consensus_output = self
                    .resolve_consensus_output(node_idx, &output.data, &checkpoint)
                    .await?;
                let final_output = match consensus_output {
                    ConsensusOutcome::Proceed(value) => value,
                    ConsensusOutcome::Suspend(suspended_context) => {
                        return Ok(Some(suspended_context));
                    }
                };

                merge_output_data(context, &final_output);
                self.set_node_status(node_idx, NodeStatus::Completed).await;
                self.emit_node_transition(node_idx, NodeTransitionPhase::Exiting, session_id)
                    .await;
                let suspend_reason = self
                    .apply_instruction(output.instruction, active_branches)
                    .await?;

                self.save_checkpoint_if_needed(
                    session_id,
                    redis_url,
                    total_steps,
                    active_branches,
                    context,
                )
                .await;

                if let Some(reason) = suspend_reason {
                    log::info!("Workflow suspended: {reason}");
                    return Ok(Some(context.clone()));
                }
                Ok(None)
            }
            Ok((node_idx, Err(error))) => {
                self.set_node_status(node_idx, NodeStatus::Failed(error.clone()))
                    .await;
                self.emit_node_transition(node_idx, NodeTransitionPhase::Failed, session_id)
                    .await;
                Err(QianjiError::Execution(error))
            }
            Err(join_err) => Err(QianjiError::Execution(format!("Task panic: {join_err}"))),
        }
    }
}
