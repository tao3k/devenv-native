use crate::QianjiEngine;
use crate::consensus::{AgentIdentity, ConsensusManager};
use crate::error::QianjiError;
use crate::scheduler::SchedulerRuntimeServices;
use crate::scheduler::{
    QianjiScheduler, RoleAvailabilityRegistry, SchedulerAgentIdentity, SchedulerExecutionPolicy,
};
use crate::swarm::engine_orchestrator::SwarmEngine;
use crate::swarm::engine_types::{
    SwarmAgentConfig, SwarmAgentReport, WorkerJoinSet, WorkerRuntimeConfig,
};
use crate::swarm::{GlobalSwarmRegistry, RemotePossessionBus};
use crate::telemetry::{SwarmEvent, unix_millis_now};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

const WORKER_BOOT_TURNS: u64 = 1;
const WORKER_SUCCESS_TURNS: u64 = 1;

impl SwarmEngine {
    pub(in crate::swarm) fn spawn_worker_task(
        &self,
        join_set: &mut WorkerJoinSet,
        identity: SwarmAgentConfig,
        context: serde_json::Value,
        runtime: WorkerRuntimeConfig,
        cancel_token: CancellationToken,
    ) {
        let engine = Arc::clone(&self.base_engine);
        join_set.spawn(Self::run_worker(
            engine,
            identity,
            context,
            runtime,
            cancel_token,
        ));
    }

    async fn run_worker(
        engine: Arc<QianjiEngine>,
        identity: SwarmAgentConfig,
        context: serde_json::Value,
        runtime: WorkerRuntimeConfig,
        cancel_token: CancellationToken,
    ) -> Result<SwarmAgentReport, QianjiError> {
        let session_id = runtime.session_id.clone();
        let role = identity.role_class.clone();

        let thread_id = format!("{:?}", std::thread::current().id());
        log::info!(
            "[THREAD_ID={thread_id}] [AGENT_ID={}] swarm worker started",
            identity.agent_id
        );
        Self::emit_pulse_event(
            runtime.pulse_emitter.as_ref(),
            SwarmEvent::SwarmHeartbeat {
                session_id: Some(session_id.clone()),
                cluster_id: runtime.cluster_id.clone(),
                agent_id: Some(identity.agent_id.clone()),
                role_class: role.clone(),
                cpu_percent: None,
                memory_bytes: None,
                timestamp_ms: unix_millis_now(),
            },
        );

        let scheduler = Self::build_worker_scheduler(&engine, &identity, &runtime);
        let (stop_tx, responder_handle) = Self::start_remote_responder(
            Arc::clone(&scheduler),
            role.clone(),
            identity.agent_id.clone(),
            runtime.remote_enabled,
            runtime.poll_interval_ms,
        );

        let run_future =
            scheduler.run_with_checkpoint(context, Some(session_id.clone()), runtime.redis_url);
        tokio::pin!(run_future);
        let run_result = tokio::select! {
            () = cancel_token.cancelled() => Err(QianjiError::Aborted(format!(
                "swarm worker '{}' cancelled by global fault broadcast",
                identity.agent_id
            ))),
            result = &mut run_future => result,
        };
        Self::stop_remote_responder(stop_tx, responder_handle).await;

        Self::build_worker_report(identity, role, run_result)
    }

    fn build_worker_scheduler(
        engine: &Arc<QianjiEngine>,
        identity: &SwarmAgentConfig,
        runtime: &WorkerRuntimeConfig,
    ) -> Arc<QianjiScheduler> {
        let redis_url: Option<&str> = runtime.redis_url.as_deref();
        let consensus_manager = redis_url.map(|url: &str| {
            Arc::new(ConsensusManager::with_agent_identity(
                url.to_string(),
                AgentIdentity {
                    id: identity.agent_id.clone(),
                    weight: identity.weight,
                },
            ))
        });
        let role_registry: Option<Arc<dyn RoleAvailabilityRegistry>> =
            redis_url.map(|url: &str| {
                Arc::new(GlobalSwarmRegistry::new(url.to_string()))
                    as Arc<dyn RoleAvailabilityRegistry>
            });
        let execution_policy = SchedulerExecutionPolicy::new()
            .with_local_proxy_delegation(runtime.allow_local_affinity_proxy);

        let scheduler_identity = SchedulerAgentIdentity::new(
            Some(identity.agent_id.clone()),
            identity.role_class.clone(),
        );
        let remote_bus = if runtime.remote_enabled {
            redis_url
                .map(std::string::ToString::to_string)
                .map(RemotePossessionBus::new)
                .map(Arc::new)
        } else {
            None
        };

        let services = SchedulerRuntimeServices {
            consensus_manager,
            remote_possession_bus: remote_bus,
            role_registry,
            cluster_id: runtime.cluster_id.clone(),
            execution_policy,
            telemetry_emitter: runtime.pulse_emitter.clone(),
        };
        Arc::new(QianjiScheduler::with_runtime_services_config(
            (**engine).clone(),
            scheduler_identity,
            services,
        ))
    }

    fn emit_pulse_event(
        pulse_emitter: Option<&Arc<dyn crate::telemetry::PulseEmitter>>,
        event: SwarmEvent,
    ) {
        let Some(emitter) = pulse_emitter.cloned() else {
            return;
        };
        std::mem::drop(tokio::spawn(async move {
            if let Err(error) = emitter.emit_pulse(event).await {
                log::debug!("swarm telemetry emission skipped: {error}");
            }
        }));
    }

    fn build_worker_report(
        identity: SwarmAgentConfig,
        role: Option<String>,
        run_result: Result<serde_json::Value, QianjiError>,
    ) -> Result<SwarmAgentReport, QianjiError> {
        let context = run_result?;
        Ok(SwarmAgentReport {
            agent_id: identity.agent_id,
            role_class: role,
            success: true,
            context: Some(context),
            error: None,
            window_turns: WORKER_BOOT_TURNS + WORKER_SUCCESS_TURNS,
            window_tool_calls: 0,
        })
    }
}
