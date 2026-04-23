use crate::consensus::ConsensusManager;
use crate::engine::QianjiEngine;
use crate::scheduler_identity::SchedulerAgentIdentity;
use crate::scheduler_policy::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
use crate::swarm::RemotePossessionBus;
use crate::telemetry::PulseEmitter;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(super) struct ConsensusCheckpointView<'a> {
    pub(super) session_id: Option<&'a str>,
    pub(super) redis_url: Option<&'a str>,
    pub(super) total_steps: u32,
    pub(super) active_branches: &'a HashSet<String>,
    pub(super) context: &'a serde_json::Value,
}

pub(super) enum ConsensusOutcome {
    Proceed(serde_json::Value),
    Suspend(serde_json::Value),
}

pub(super) const EXTERNAL_PROGRESS_WAIT_MS: u64 = 200;
pub(super) const EXTERNAL_PROGRESS_TIMEOUT_MS: u64 = 30_000;
pub(super) const REMOTE_POSSESSION_REQUEST_TTL_SECONDS: u64 = 120;
pub(super) const REMOTE_POSSESSION_MAX_WAIT_MS: u64 = 30_000;

pub(super) enum RemoteDelegationOutcome {
    Noop,
    Progressed,
    Suspend(serde_json::Value),
}

/// Runtime dependency bundle for scheduler execution.
#[derive(Clone, Default)]
pub struct SchedulerRuntimeServices {
    /// Optional manager for distributed consensus voting.
    pub consensus_manager: Option<Arc<ConsensusManager>>,
    /// Optional remote possession transport for cross-cluster delegation.
    pub remote_possession_bus: Option<Arc<RemotePossessionBus>>,
    /// Optional global role availability registry used by affinity failover.
    pub role_registry: Option<Arc<dyn RoleAvailabilityRegistry>>,
    /// Optional local cluster id override.
    pub cluster_id: Option<String>,
    /// Execution policy for role affinity and local proxy behavior.
    pub execution_policy: SchedulerExecutionPolicy,
    /// Optional non-blocking telemetry emitter for swarm pulse events.
    pub telemetry_emitter: Option<Arc<dyn PulseEmitter>>,
}

/// Drives the parallel execution of the Qianji Box mechanisms.
pub struct QianjiScheduler {
    /// Thread-safe access to the underlying graph.
    pub(super) engine: Arc<RwLock<QianjiEngine>>,
    /// Maximum total execution steps to prevent runaway loops.
    pub(super) max_total_steps: u32,
    /// Optional manager for distributed consensus voting.
    pub(super) consensus_manager: Option<Arc<ConsensusManager>>,
    /// Optional remote possession transport for cross-cluster delegation.
    pub(super) remote_possession_bus: Option<Arc<RemotePossessionBus>>,
    /// Optional global role availability registry used by affinity failover.
    pub(super) role_registry: Option<Arc<dyn RoleAvailabilityRegistry>>,
    /// Local cluster id used to avoid self-delegation loops.
    pub(super) cluster_id: String,
    /// Runtime execution identity used by role-aware scheduling.
    pub(super) execution_identity: SchedulerAgentIdentity,
    /// Runtime execution policy for affinity and local delegation.
    pub(super) execution_policy: SchedulerExecutionPolicy,
    /// Optional non-blocking telemetry emitter for swarm pulse events.
    pub(super) telemetry_emitter: Option<Arc<dyn PulseEmitter>>,
}

impl QianjiScheduler {
    /// Creates a new scheduler for the given engine.
    #[must_use]
    pub fn new(engine: QianjiEngine) -> Self {
        Self::with_consensus_manager(engine, None)
    }

    /// Creates a new scheduler with optional distributed consensus manager.
    #[must_use]
    pub fn with_consensus_manager(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
    ) -> Self {
        Self::with_runtime_services_config(
            engine,
            SchedulerAgentIdentity::from_env(),
            SchedulerRuntimeServices {
                consensus_manager,
                ..Default::default()
            },
        )
    }

    /// Creates a scheduler with optional consensus manager and explicit identity.
    #[must_use]
    pub fn with_consensus_manager_and_identity(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
        execution_identity: SchedulerAgentIdentity,
    ) -> Self {
        Self::with_runtime_services_config(
            engine,
            execution_identity,
            SchedulerRuntimeServices {
                consensus_manager,
                ..Default::default()
            },
        )
    }

    /// Creates a scheduler with explicit runtime services and inferred defaults.
    #[must_use]
    pub fn with_runtime_services(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
        remote_possession_bus: Option<Arc<RemotePossessionBus>>,
        cluster_id: Option<String>,
        execution_identity: SchedulerAgentIdentity,
    ) -> Self {
        Self::with_runtime_services_config(
            engine,
            execution_identity,
            SchedulerRuntimeServices {
                consensus_manager,
                remote_possession_bus,
                cluster_id,
                ..Default::default()
            },
        )
    }

    /// Creates a scheduler with the full runtime dependency bundle.
    #[must_use]
    pub fn with_runtime_services_config(
        engine: QianjiEngine,
        execution_identity: SchedulerAgentIdentity,
        services: SchedulerRuntimeServices,
    ) -> Self {
        let cluster_id = services
            .cluster_id
            .or_else(|| std::env::var("CLUSTER_ID").ok())
            .unwrap_or_else(|| "local_cluster".to_string());

        Self {
            engine: Arc::new(RwLock::new(engine)),
            max_total_steps: 1000,
            consensus_manager: services.consensus_manager,
            remote_possession_bus: services.remote_possession_bus,
            role_registry: services.role_registry,
            cluster_id,
            execution_identity,
            execution_policy: services.execution_policy,
            telemetry_emitter: services.telemetry_emitter,
        }
    }
}
