use super::engine_types::{
    SwarmAgentConfig, SwarmExecutionOptions, SwarmExecutionReport, WorkerJoinSet,
    WorkerRuntimeConfig, generate_swarm_session_id,
};
use crate::QianjiEngine;
use crate::error::QianjiError;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Orchestrates one flow with multiple isolated worker schedulers.
pub struct SwarmEngine {
    pub(super) base_engine: Arc<QianjiEngine>,
}

impl SwarmEngine {
    /// Creates a new swarm engine from a compiled base engine.
    #[must_use]
    pub fn new(base_engine: QianjiEngine) -> Self {
        Self {
            base_engine: Arc::new(base_engine),
        }
    }

    /// Executes one workflow across multiple worker identities concurrently.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when all workers fail to produce a successful context
    /// or when worker task orchestration fails.
    pub async fn execute_swarm(
        &self,
        initial_context: serde_json::Value,
        identities: Vec<SwarmAgentConfig>,
        options: SwarmExecutionOptions,
    ) -> Result<SwarmExecutionReport, QianjiError> {
        if identities.is_empty() {
            return Err(QianjiError::Execution(
                "Swarm execution requires at least one agent identity".to_string(),
            ));
        }

        let runtime = WorkerRuntimeConfig {
            session_id: options.session_id.unwrap_or_else(generate_swarm_session_id),
            redis_url: options.redis_url,
            cluster_id: options.cluster_id,
            remote_enabled: options.enable_remote_possession,
            poll_interval_ms: options.possession_poll_interval_ms.max(100),
            allow_local_affinity_proxy: options.allow_local_affinity_proxy,
            pulse_emitter: options.pulse_emitter,
        };

        let cancel_token = CancellationToken::new();
        let mut join_set = WorkerJoinSet::new();
        for identity in identities {
            self.spawn_worker_task(
                &mut join_set,
                identity,
                initial_context.clone(),
                runtime.clone(),
                cancel_token.clone(),
            );
        }

        let workers = Self::collect_worker_reports(&mut join_set, &cancel_token).await?;
        let final_context = Self::select_final_context(&workers)?;
        Ok(SwarmExecutionReport {
            session_id: runtime.session_id,
            final_context,
            workers,
        })
    }
}
