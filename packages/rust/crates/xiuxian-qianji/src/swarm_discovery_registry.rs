//! Swarm discovery registry branch for Valkey-backed cluster membership.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;

use super::discovery_model::{ClusterNodeIdentity, ClusterNodeRecord};

#[path = "swarm/discovery/registry/connection.rs"]
mod connection;
#[path = "swarm/discovery/registry/discover.rs"]
mod discover;
#[path = "swarm/discovery/registry/heartbeat.rs"]
mod heartbeat;
#[path = "swarm/discovery/registry/keys.rs"]
mod keys;
#[path = "swarm/discovery/registry/payload.rs"]
mod payload;

/// Valkey-backed global discovery registry.
pub struct GlobalSwarmRegistry {
    pub(super) redis_url: String,
    pub(super) connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    pub(super) reconnect_lock: Arc<Mutex<()>>,
}

impl GlobalSwarmRegistry {
    /// Creates a discovery registry using the provided Valkey URL.
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Discovers all live nodes from the global registry.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub async fn discover_all(&self) -> Result<Vec<ClusterNodeRecord>> {
        self.discover_all_impl().await
    }

    /// Discovers live nodes by role class.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub async fn discover_by_role(&self, role_class: &str) -> Result<Vec<ClusterNodeRecord>> {
        self.discover_by_role_impl(role_class).await
    }

    /// Picks one live remote node matching a role class.
    ///
    /// Returns `None` when no candidate is available.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub async fn pick_candidate(
        &self,
        role_class: &str,
        exclude_cluster_id: Option<&str>,
    ) -> Result<Option<ClusterNodeRecord>> {
        self.pick_candidate_impl(role_class, exclude_cluster_id)
            .await
    }

    /// Writes one heartbeat lease into the global registry.
    ///
    /// # Errors
    ///
    /// Returns an error when input fields are invalid or any Valkey command fails.
    pub async fn heartbeat(
        &self,
        identity: &ClusterNodeIdentity,
        metadata: &serde_json::Value,
        ttl_seconds: u64,
    ) -> Result<()> {
        self.heartbeat_impl(identity, metadata, ttl_seconds).await
    }

    /// Spawns a background heartbeat loop for one node identity.
    ///
    /// # Errors
    ///
    /// Returns an error when `ttl_seconds` or `interval` is invalid.
    pub fn spawn_heartbeat_loop(
        self: Arc<Self>,
        identity: ClusterNodeIdentity,
        metadata: serde_json::Value,
        ttl_seconds: u64,
        interval: Duration,
    ) -> Result<tokio::task::JoinHandle<()>> {
        self.spawn_heartbeat_loop_impl(identity, metadata, ttl_seconds, interval)
    }
}
