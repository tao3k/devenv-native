use anyhow::{Result, anyhow};
use tokio::time::Duration;

use super::GlobalSwarmRegistry;
use super::keys::node_key;
use super::payload::heartbeat_payload;
use crate::swarm::discovery_model::ClusterNodeIdentity;
use crate::swarm::discovery_util::REGISTRY_INDEX_KEY;

impl GlobalSwarmRegistry {
    /// Writes one heartbeat lease into the global registry.
    ///
    /// # Errors
    ///
    /// Returns an error when input fields are invalid or any Valkey command fails.
    pub(super) async fn heartbeat_impl(
        &self,
        identity: &ClusterNodeIdentity,
        metadata: &serde_json::Value,
        ttl_seconds: u64,
    ) -> Result<()> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        let sanitized = identity.clone().sanitize();
        let key = node_key(&sanitized);
        let fields = heartbeat_payload(&sanitized, metadata)?;

        for (field, value) in &fields {
            let _: i64 = self
                .run_command("swarm_registry_hset", || {
                    let mut command = redis::cmd("HSET");
                    command.arg(&key).arg(field).arg(value);
                    command
                })
                .await?;
        }

        let _: bool = self
            .run_command("swarm_registry_expire", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&key).arg(ttl_seconds);
                command
            })
            .await?;
        let _: i64 = self
            .run_command("swarm_registry_index_add", || {
                let mut command = redis::cmd("SADD");
                command.arg(REGISTRY_INDEX_KEY).arg(&key);
                command
            })
            .await?;
        Ok(())
    }

    /// Spawns a background heartbeat loop for one node identity.
    ///
    /// # Errors
    ///
    /// Returns an error when `ttl_seconds` or `interval` is invalid.
    pub(super) fn spawn_heartbeat_loop_impl(
        self: std::sync::Arc<Self>,
        identity: ClusterNodeIdentity,
        metadata: serde_json::Value,
        ttl_seconds: u64,
        interval: Duration,
    ) -> Result<tokio::task::JoinHandle<()>> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        if interval.is_zero() {
            return Err(anyhow!("heartbeat interval must be > 0"));
        }
        let min_ttl = interval.as_secs().saturating_mul(2);
        if ttl_seconds <= min_ttl {
            return Err(anyhow!(
                "ttl_seconds must be at least 2x interval (ttl={ttl_seconds}, interval_secs={})",
                interval.as_secs()
            ));
        }

        let handle = tokio::spawn(async move {
            loop {
                if let Err(error) = self.heartbeat_impl(&identity, &metadata, ttl_seconds).await {
                    log::warn!("swarm heartbeat failed for {}: {error}", identity.agent_id);
                }
                tokio::time::sleep(interval).await;
            }
        });
        Ok(handle)
    }
}
