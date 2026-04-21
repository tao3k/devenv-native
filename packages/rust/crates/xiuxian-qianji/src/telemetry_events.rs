use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default Valkey channel used for swarm pulse telemetry.
pub const DEFAULT_PULSE_CHANNEL: &str = "xiuxian:swarm:pulse";

/// Node transition stage in scheduler execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeTransitionPhase {
    /// Scheduler just queued node execution.
    Entering,
    /// Scheduler completed node execution successfully.
    Exiting,
    /// Scheduler marked node execution as failed.
    Failed,
}

/// Consensus lifecycle status used by pulse telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusStatus {
    /// Vote submitted but quorum is not met yet.
    Pending,
    /// Quorum reached and output hash agreed.
    Agreed,
    /// Consensus gate failed due to timeout/conflict.
    Failed,
}

/// Typed swarm telemetry event envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SwarmEvent {
    /// Lightweight worker heartbeat used for liveness/cluster monitoring.
    SwarmHeartbeat {
        /// Logical swarm session identifier.
        session_id: Option<String>,
        /// Cluster id where the worker is running.
        cluster_id: Option<String>,
        /// Worker agent id when available.
        agent_id: Option<String>,
        /// Worker role class when available.
        role_class: Option<String>,
        /// Optional CPU usage sampled by caller.
        cpu_percent: Option<f32>,
        /// Optional resident memory usage sampled by caller.
        memory_bytes: Option<u64>,
        /// Event timestamp in UNIX milliseconds.
        timestamp_ms: u64,
    },
    /// Scheduler node lifecycle transition.
    NodeTransition {
        /// Logical swarm session identifier.
        session_id: Option<String>,
        /// Worker agent id when available.
        agent_id: Option<String>,
        /// Worker role class when available.
        role_class: Option<String>,
        /// Node identifier from compiled flow graph.
        node_id: String,
        /// Transition phase emitted by scheduler loop.
        phase: NodeTransitionPhase,
        /// Event timestamp in UNIX milliseconds.
        timestamp_ms: u64,
    },
    /// Consensus state signal for observability consumers.
    ConsensusSpike {
        /// Logical swarm session identifier.
        session_id: String,
        /// Node identifier from compiled flow graph.
        node_id: String,
        /// Current consensus status.
        status: ConsensusStatus,
        /// Optional progress ratio in range `[0.0, 1.0]`.
        progress: Option<f32>,
        /// Optional target ratio in range `[0.0, 1.0]`.
        target: Option<f32>,
        /// Event timestamp in UNIX milliseconds.
        timestamp_ms: u64,
    },
    /// Event fired when one manifestation artifact is produced.
    EvolutionBirth {
        /// Logical swarm session identifier.
        session_id: Option<String>,
        /// Role id that produced the manifestation.
        role_id: Option<String>,
        /// Relative/absolute manifestation path.
        manifestation_path: String,
        /// Event timestamp in UNIX milliseconds.
        timestamp_ms: u64,
    },
    /// Affinity failover warning when local proxy delegation is activated.
    AffinityAlert {
        /// Logical swarm session identifier.
        session_id: Option<String>,
        /// Node identifier from compiled flow graph.
        node_id: String,
        /// Role required by node affinity.
        required_role: String,
        /// Agent id that served as local proxy.
        proxy_agent_id: Option<String>,
        /// Role class that served as local proxy.
        proxy_role: Option<String>,
        /// Event timestamp in UNIX milliseconds.
        timestamp_ms: u64,
    },
    /// Cognitive supervision metrics emitted during LLM streaming.
    CognitivePulse {
        /// Logical swarm session identifier.
        session_id: Option<String>,
        /// Node identifier from compiled flow graph.
        node_id: String,
        /// Current coherence score (0.0 to 1.0).
        coherence: f32,
        /// Whether early halt was triggered.
        early_halt_triggered: bool,
        /// Cognitive distribution metrics.
        distribution: CognitiveDistributionMetrics,
        /// Event timestamp in UNIX milliseconds.
        timestamp_ms: u64,
    },
}

/// Cognitive distribution metrics for telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CognitiveDistributionMetrics {
    /// Meta-cognitive dimension score.
    pub meta: f32,
    /// Operational dimension score.
    pub operational: f32,
    /// Epistemic dimension score.
    pub epistemic: f32,
    /// Instrumental dimension score.
    pub instrumental: f32,
    /// Balance ratio between dimensions.
    pub balance: f32,
    /// Uncertainty ratio.
    pub uncertainty_ratio: f32,
}

/// Returns current UNIX timestamp in milliseconds.
#[must_use]
pub fn unix_millis_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
