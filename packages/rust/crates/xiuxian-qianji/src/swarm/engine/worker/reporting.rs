use crate::error::QianjiError;
use crate::swarm::engine_orchestrator::SwarmEngine;
use crate::swarm::engine_types::{SwarmAgentReport, WorkerJoinSet};
use tokio_util::sync::CancellationToken;

impl SwarmEngine {
    pub(in crate::swarm) async fn collect_worker_reports(
        join_set: &mut WorkerJoinSet,
        cancel_token: &CancellationToken,
    ) -> Result<Vec<SwarmAgentReport>, QianjiError> {
        let mut workers = Vec::new();
        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok(Ok(report)) => workers.push(report),
                Ok(Err(error)) => {
                    cancel_token.cancel();
                    return Err(error);
                }
                Err(join_error) => {
                    cancel_token.cancel();
                    return Err(QianjiError::Execution(format!(
                        "swarm worker join panic: {join_error}"
                    )));
                }
            }
        }
        Ok(workers)
    }

    pub(in crate::swarm) fn select_final_context(
        workers: &[SwarmAgentReport],
    ) -> Result<serde_json::Value, QianjiError> {
        workers
            .iter()
            .find_map(|worker| {
                if worker.success {
                    worker.context.clone()
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                let errors = workers
                    .iter()
                    .filter_map(|worker| worker.error.as_deref())
                    .collect::<Vec<_>>()
                    .join(" | ");
                QianjiError::Execution(format!("all swarm workers failed: {errors}"))
            })
    }
}
