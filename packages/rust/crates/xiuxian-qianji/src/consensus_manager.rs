//! Consensus manager branch for Valkey-backed node-vote coordination.

use super::models::{AgentIdentity, AgentVote, ConsensusPolicy, ConsensusResult};
use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, timeout};

#[path = "consensus/manager/connection.rs"]
mod connection;
#[path = "consensus/manager/keys.rs"]
mod keys;
#[path = "consensus/manager/time.rs"]
mod time;
#[path = "consensus_manager_voting.rs"]
mod voting;

use keys::VoteKeys;
use time::current_unix_millis;

const CONSENSUS_VOTE_TTL_SECONDS: u64 = 300;

/// Orchestrates distributed voting via Valkey.
pub struct ConsensusManager {
    redis_url: String,
    agent_identity: AgentIdentity,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl ConsensusManager {
    /// Creates a consensus manager backed by the given Valkey/Redis URL.
    ///
    /// Agent identity defaults to:
    /// - `AGENT_ID` env or `local_agent`
    /// - `AGENT_WEIGHT` env or `1.0`
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self::with_agent_identity(redis_url, AgentIdentity::from_env())
    }

    /// Creates a consensus manager with explicit agent identity.
    #[must_use]
    pub fn with_agent_identity(redis_url: String, agent_identity: AgentIdentity) -> Self {
        Self {
            redis_url,
            agent_identity,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Submits one vote and returns consensus verdict in one Rust-side flow.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or vote serialization fails.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub async fn submit_vote(
        &self,
        session_id: &str,
        node_id: &str,
        output_hash: String,
        policy: &ConsensusPolicy,
    ) -> Result<ConsensusResult> {
        self.submit_vote_with_payload(session_id, node_id, output_hash, None, policy)
            .await
    }

    /// Submits one vote with optional serialized output payload.
    ///
    /// This method is used by scheduler-level consensus gates so that non-winning
    /// agent processes can materialize the agreed payload without recomputation.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or vote serialization fails.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    /// Positional boundary: this compatibility API keeps the established public call shape.
    pub async fn submit_vote_with_payload(
        &self,
        session_id: &str,
        node_id: &str,
        output_hash: String,
        output_payload: Option<&str>,
        policy: &ConsensusPolicy,
    ) -> Result<ConsensusResult> {
        let vote = AgentVote {
            agent_id: self.agent_identity.id.clone(),
            output_hash,
            weight: self.agent_identity.weight,
            timestamp_ms: u128::from(current_unix_millis()),
        };
        self.submit_vote_payload(session_id, node_id, vote, output_payload, policy)
            .await
    }

    /// Returns the stored output payload for an agreed hash, if available.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey lookup fails.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub async fn get_output_payload(
        &self,
        session_id: &str,
        node_id: &str,
        output_hash: &str,
    ) -> Result<Option<String>> {
        let keys = VoteKeys::new(session_id, node_id);
        self.run_command("consensus_get_output_payload", || {
            let mut command = redis::cmd("HGET");
            command.arg(&keys.output_payloads).arg(output_hash);
            command
        })
        .await
    }

    /// Waits asynchronously for quorum result published for one node.
    ///
    /// Returns `Ok(Some(hash))` when winner hash is observed, `Ok(None)` on timeout.
    ///
    /// # Errors
    ///
    /// Returns an error when pub/sub connection or channel operations fail.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub async fn wait_for_quorum(
        &self,
        session_id: &str,
        node_id: &str,
        max_wait: Duration,
    ) -> Result<Option<String>> {
        let keys = VoteKeys::new(session_id, node_id);
        if let Some(winner) = self.current_winner(keys.winner_marker.as_str()).await? {
            return Ok(Some(winner));
        }

        let client = redis::Client::open(self.redis_url.as_str())
            .context("Failed to connect to Valkey pubsub for consensus wait")?;
        let mut pubsub = client.get_async_pubsub().await?;
        let channel = keys.quorum_channel();
        pubsub.subscribe(channel).await?;
        let mut stream = pubsub.on_message();

        match timeout(max_wait, async {
            while let Some(message) = stream.next().await {
                let payload: String = message.get_payload()?;
                if !payload.trim().is_empty() {
                    return Ok(payload);
                }
            }
            Err(anyhow!("consensus pubsub stream closed unexpectedly"))
        })
        .await
        {
            Ok(inner) => inner.map(Some),
            Err(_elapsed) => Ok(None),
        }
    }
}
