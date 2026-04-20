use std::collections::HashMap;

use anyhow::Result;
use rand::seq::SliceRandom;

use super::GlobalSwarmRegistry;
use crate::swarm::discovery_model::ClusterNodeRecord;
use crate::swarm::discovery_parse::{parse_record, role_matches};
use crate::swarm::discovery_util::{REGISTRY_INDEX_KEY, normalize_optional_text};

impl GlobalSwarmRegistry {
    /// Discovers all live nodes from the global registry.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub(super) async fn discover_all_impl(&self) -> Result<Vec<ClusterNodeRecord>> {
        self.discover(Some("*")).await
    }

    /// Discovers live nodes by role class.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub(super) async fn discover_by_role_impl(
        &self,
        role_class: &str,
    ) -> Result<Vec<ClusterNodeRecord>> {
        let normalized = role_class.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(Vec::new());
        }
        self.discover(Some(normalized.as_str())).await
    }

    /// Picks one live remote node matching a role class.
    ///
    /// Returns `None` when no candidate is available.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub(super) async fn pick_candidate_impl(
        &self,
        role_class: &str,
        exclude_cluster_id: Option<&str>,
    ) -> Result<Option<ClusterNodeRecord>> {
        let mut records = self.discover_by_role_impl(role_class).await?;
        if let Some(exclude) = normalize_optional_text(exclude_cluster_id.map(ToString::to_string))
        {
            records.retain(|record| record.identity.cluster_id != exclude);
        }
        let mut rng = rand::thread_rng();
        Ok(records.choose(&mut rng).cloned())
    }

    async fn discover(&self, role_filter: Option<&str>) -> Result<Vec<ClusterNodeRecord>> {
        let keys: Vec<String> = self
            .run_command("swarm_registry_index_members", || {
                let mut command = redis::cmd("SMEMBERS");
                command.arg(REGISTRY_INDEX_KEY);
                command
            })
            .await?;
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();
        let mut stale = Vec::new();
        for key in keys {
            let fields: HashMap<String, String> = self
                .run_command("swarm_registry_hgetall", || {
                    let mut command = redis::cmd("HGETALL");
                    command.arg(&key);
                    command
                })
                .await?;
            if fields.is_empty() {
                stale.push(key);
                continue;
            }

            if let Some(record) = parse_record(key, &fields)
                && role_matches(role_filter, &record.identity.role_class)
            {
                records.push(record);
            }
        }

        if !stale.is_empty() {
            for key in stale {
                let _: i64 = self
                    .run_command("swarm_registry_prune_stale_index", || {
                        let mut command = redis::cmd("SREM");
                        command.arg(REGISTRY_INDEX_KEY).arg(&key);
                        command
                    })
                    .await?;
            }
        }

        Ok(records)
    }
}
