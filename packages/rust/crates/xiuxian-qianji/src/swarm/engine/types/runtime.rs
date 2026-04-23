use crate::error::QianjiError;
use crate::telemetry::PulseEmitter;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::swarm::engine_types::SwarmAgentReport;

pub(in crate::swarm) type WorkerJoinSet =
    tokio::task::JoinSet<Result<SwarmAgentReport, QianjiError>>;

#[derive(Debug, Clone)]
pub(in crate::swarm) struct WorkerRuntimeConfig {
    pub(in crate::swarm) session_id: String,
    pub(in crate::swarm) redis_url: Option<String>,
    pub(in crate::swarm) cluster_id: Option<String>,
    pub(in crate::swarm) remote_enabled: bool,
    pub(in crate::swarm) poll_interval_ms: u64,
    pub(in crate::swarm) allow_local_affinity_proxy: bool,
    pub(in crate::swarm) pulse_emitter: Option<Arc<dyn PulseEmitter>>,
}

pub(in crate::swarm) fn generate_swarm_session_id() -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random_suffix: u64 = rand::random();
    format!("swarm_{now_ms}_{random_suffix:x}")
}
