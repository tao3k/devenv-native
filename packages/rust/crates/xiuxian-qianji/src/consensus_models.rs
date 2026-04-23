use serde::{Deserialize, Serialize};

/// Defines the strategy for achieving agreement across distributed agent instances.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusMode {
    /// Agreement by simple majority (> 50%).
    Majority,
    /// Absolute agreement (100%).
    Unanimous,
    /// Agreement based on the sum of agent weights.
    Weighted,
}

/// Configuration for a consensus gate on a specific node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusPolicy {
    /// Consensus strategy mode.
    pub mode: ConsensusMode,
    /// Minimum number of agents required to reach quorum.
    pub min_agents: usize,
    /// Timeout for achieving consensus before node failure.
    pub timeout_ms: u64,
    /// Weight threshold required when in `Weighted` mode.
    pub weight_threshold: f32,
}

impl Default for ConsensusPolicy {
    fn default() -> Self {
        Self {
            mode: ConsensusMode::Majority,
            min_agents: 1,
            timeout_ms: 30000, // 30 seconds
            weight_threshold: 0.5,
        }
    }
}

/// Immutable identity injected into `ConsensusManager`.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentIdentity {
    /// Stable agent identifier used as vote field key.
    pub id: String,
    /// Default vote weight for this agent.
    pub weight: f32,
}

impl AgentIdentity {
    #[must_use]
    pub(crate) fn from_env() -> Self {
        let id = std::env::var("AGENT_ID").unwrap_or_else(|_| "local_agent".to_string());
        let weight = std::env::var("AGENT_WEIGHT")
            .ok()
            .and_then(|raw| raw.trim().parse::<f32>().ok())
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(1.0);
        Self { id, weight }
    }
}

/// A single vote cast by an agent instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentVote {
    /// Unique agent identifier that cast this vote.
    pub agent_id: String,
    /// Hash of the serialized node output the agent proposes.
    pub output_hash: String,
    /// Vote weight for weighted consensus mode.
    pub weight: f32,
    /// Vote timestamp in milliseconds since UNIX epoch.
    pub timestamp_ms: u128,
}

/// Result of a consensus check.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusResult {
    /// Quorum reached with the agreed-upon hash.
    Agreed(String),
    /// No agreement reached yet (waiting for more votes).
    Pending,
    /// Consensus failed (too many conflicting votes or timeout).
    Failed(String),
}
