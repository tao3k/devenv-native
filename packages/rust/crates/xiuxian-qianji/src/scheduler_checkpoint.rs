//! Valkey (Redis) checkpointing for Qianji workflows.
//! Enables interrupting and resuming workflows seamlessly.

use crate::contracts::NodeStatus;
use crate::error::QianjiError;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// State snapshot containing the exact status of a running Qianji workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QianjiStateSnapshot {
    /// Associated session/thread ID.
    pub session_id: String,
    /// Total execution steps taken so far.
    pub total_steps: u32,
    /// Branches that have been selected/activated.
    pub active_branches: HashSet<String>,
    /// Accumulated context data.
    pub context: serde_json::Value,
    /// Mapping of node ID to its current execution status.
    pub node_statuses: HashMap<String, NodeStatus>,
}

impl QianjiStateSnapshot {
    /// Formats the Redis key for a given session.
    #[must_use]
    pub fn redis_key(session_id: &str) -> String {
        format!("xq:qianji:checkpoint:{session_id}")
    }

    /// Load a state snapshot from Valkey/Redis.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError::CheckpointError`] when redis connectivity,
    /// key lookup, or JSON deserialization fails.
    pub async fn load(session_id: &str, redis_url: &str) -> Result<Option<Self>, QianjiError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;
        let mut con = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;

        let key = Self::redis_key(session_id);
        let data: Option<String> = con
            .get(&key)
            .await
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;

        match data {
            Some(json_str) => {
                let snapshot = serde_json::from_str(&json_str)
                    .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    /// Save the current state snapshot to Valkey/Redis.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError::CheckpointError`] when redis connectivity,
    /// key write, or JSON serialization fails.
    pub async fn save(&self, redis_url: &str) -> Result<(), QianjiError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;
        let mut con = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;

        let key = Self::redis_key(&self.session_id);
        let json_str =
            serde_json::to_string(self).map_err(|e| QianjiError::CheckpointError(e.to_string()))?;

        // Expire checkpoint after 7 days (604800 seconds)
        let _: () = con
            .set_ex(&key, json_str, 604_800)
            .await
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;

        Ok(())
    }

    /// Delete a checkpoint from Valkey/Redis.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError::CheckpointError`] when redis connectivity
    /// or delete command execution fails.
    pub async fn delete(session_id: &str, redis_url: &str) -> Result<(), QianjiError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;
        let mut con = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;

        let key = Self::redis_key(session_id);
        let _: () = con
            .del(&key)
            .await
            .map_err(|e| QianjiError::CheckpointError(e.to_string()))?;
        Ok(())
    }
}
