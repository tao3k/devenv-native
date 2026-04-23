use crate::QianjiEngine;
use crate::consensus::{AgentIdentity, ConsensusManager};
use crate::scheduler::{
    QianjiScheduler, RoleAvailabilityRegistry, SchedulerAgentIdentity, SchedulerExecutionPolicy,
    SchedulerRuntimeServices,
};
use crate::swarm::engine_orchestrator::SwarmEngine;
use crate::swarm::engine_types::{SwarmAgentConfig, WorkerRuntimeConfig};
use crate::swarm::{GlobalSwarmRegistry, RemotePossessionBus};
use std::sync::Arc;

impl SwarmEngine {
    pub(in crate::swarm::engine::worker) fn build_worker_scheduler(
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
}
