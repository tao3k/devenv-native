//! Valkey-backed memory state persistence.

use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use redis::Commands;

use super::{
    MemoryStateStore, default_valkey_recall_feedback_hash_key, default_valkey_state_hash_keys,
};
use crate::{Episode, EpisodeStore, MemoryStateSnapshot, normalize_feedback_bias};

/// Valkey-backed memory state store using hash-based persistence.
///
/// - Episodes are stored in a hash keyed by episode id.
/// - Q-values are stored in a hash keyed by episode id.
/// - Legacy single-blob snapshots are still readable for migration.
pub struct ValkeyMemoryStateStore {
    client: redis::Client,
    legacy_snapshot_key: String,
    episodes_hash_key: String,
    q_values_hash_key: String,
    recall_feedback_hash_key: String,
    strict_startup: bool,
}

impl ValkeyMemoryStateStore {
    /// Create a Valkey memory state store.
    ///
    /// # Errors
    ///
    /// Returns an error if `redis_url` is invalid.
    pub fn new(
        redis_url: impl AsRef<str>,
        key: impl Into<String>,
        strict_startup: bool,
    ) -> Result<Self> {
        let redis_url = redis_url.as_ref();
        let client = redis::Client::open(redis_url)
            .with_context(|| format!("invalid redis url for memory persistence: {redis_url}"))?;
        let legacy_snapshot_key = key.into();
        let (episodes_hash_key, q_values_hash_key) =
            default_valkey_state_hash_keys(&legacy_snapshot_key);
        let recall_feedback_hash_key =
            default_valkey_recall_feedback_hash_key(&legacy_snapshot_key);
        Ok(Self {
            client,
            legacy_snapshot_key,
            episodes_hash_key,
            q_values_hash_key,
            recall_feedback_hash_key,
            strict_startup,
        })
    }

    fn decode_hash_snapshot(
        episodes_raw: HashMap<String, String>,
        q_values: HashMap<String, f32>,
        recall_feedback_bias_by_scope: HashMap<String, f32>,
    ) -> Result<Option<MemoryStateSnapshot>> {
        if episodes_raw.is_empty()
            && q_values.is_empty()
            && recall_feedback_bias_by_scope.is_empty()
        {
            return Ok(None);
        }

        let mut episodes: Vec<Episode> = Vec::with_capacity(episodes_raw.len());
        for (episode_id, payload) in episodes_raw {
            let episode: Episode = serde_json::from_str(&payload).with_context(|| {
                format!("failed to decode valkey episode payload for episode id `{episode_id}`")
            })?;
            if episode.id != episode_id {
                bail!(
                    "valkey episode hash key mismatch: key `{episode_id}` but payload id `{}`",
                    episode.id
                );
            }
            episodes.push(episode);
        }
        episodes.sort_by_key(|episode| episode.created_at);

        Ok(Some(MemoryStateSnapshot {
            episodes,
            q_values,
            recall_feedback_bias_by_scope: recall_feedback_bias_by_scope
                .into_iter()
                .map(|(scope, value)| (scope, normalize_feedback_bias(value)))
                .collect(),
        }))
    }

    fn encode_episode_pairs(snapshot: &MemoryStateSnapshot) -> Result<Vec<(String, String)>> {
        snapshot
            .episodes
            .iter()
            .map(|episode| {
                let payload = serde_json::to_string(episode).with_context(|| {
                    format!(
                        "failed to encode valkey episode payload for episode id `{}`",
                        episode.id
                    )
                })?;
                Ok((episode.id.clone(), payload))
            })
            .collect()
    }
}

impl MemoryStateStore for ValkeyMemoryStateStore {
    fn backend_name(&self) -> &'static str {
        "valkey"
    }

    fn strict_startup(&self) -> bool {
        self.strict_startup
    }

    fn load(&self, store: &mut EpisodeStore) -> Result<()> {
        let mut connection = self
            .client
            .get_connection()
            .context("failed to open valkey connection for memory load")?;

        let episodes_raw: HashMap<String, String> = connection
            .hgetall(&self.episodes_hash_key)
            .context("failed to read valkey episode hash state")?;
        let q_values: HashMap<String, f32> = connection
            .hgetall(&self.q_values_hash_key)
            .context("failed to read valkey q-value hash state")?;
        let recall_feedback_bias_by_scope: HashMap<String, f32> = connection
            .hgetall(&self.recall_feedback_hash_key)
            .context("failed to read valkey recall-feedback hash state")?;

        if let Some(snapshot) =
            Self::decode_hash_snapshot(episodes_raw, q_values, recall_feedback_bias_by_scope)?
        {
            store.restore_snapshot(snapshot);
            return Ok(());
        }

        let payload: Option<String> = connection
            .get(&self.legacy_snapshot_key)
            .context("failed to read legacy valkey memory snapshot")?;
        let Some(payload) = payload else {
            return Ok(());
        };
        let snapshot: MemoryStateSnapshot = serde_json::from_str(&payload)
            .context("failed to decode legacy valkey memory snapshot")?;
        store.restore_snapshot(snapshot.clone());

        // Best-effort migration to hash keys; keep legacy payload for backward compatibility.
        let episode_pairs = Self::encode_episode_pairs(&snapshot)?;
        if !episode_pairs.is_empty() {
            connection
                .hset_multiple::<_, _, _, ()>(&self.episodes_hash_key, &episode_pairs)
                .context("failed to migrate legacy episodes into valkey hash state")?;
        }
        let q_value_pairs: Vec<(String, f32)> = snapshot.q_values.into_iter().collect();
        if !q_value_pairs.is_empty() {
            connection
                .hset_multiple::<_, _, _, ()>(&self.q_values_hash_key, &q_value_pairs)
                .context("failed to migrate legacy q-values into valkey hash state")?;
        }
        Ok(())
    }

    fn save(&self, store: &EpisodeStore) -> Result<()> {
        let snapshot = store.snapshot();
        let episode_pairs = Self::encode_episode_pairs(&snapshot)?;
        let q_value_pairs: Vec<(String, f32)> = snapshot.q_values.clone().into_iter().collect();
        let recall_feedback_pairs: Vec<(String, f32)> = snapshot
            .recall_feedback_bias_by_scope
            .into_iter()
            .map(|(scope, value)| (scope, normalize_feedback_bias(value)))
            .collect();
        let mut connection = self
            .client
            .get_connection()
            .context("failed to open valkey connection for memory save")?;
        if !episode_pairs.is_empty() {
            connection
                .hset_multiple::<_, _, _, ()>(&self.episodes_hash_key, &episode_pairs)
                .context("failed to write valkey episode hash state")?;
        }
        if !q_value_pairs.is_empty() {
            connection
                .hset_multiple::<_, _, _, ()>(&self.q_values_hash_key, &q_value_pairs)
                .context("failed to write valkey q-value hash state")?;
        }
        if !recall_feedback_pairs.is_empty() {
            connection
                .hset_multiple::<_, _, _, ()>(
                    &self.recall_feedback_hash_key,
                    &recall_feedback_pairs,
                )
                .context("failed to write valkey recall-feedback hash state")?;
        }
        Ok(())
    }

    fn update_q_atomic(&self, episode_id: &str, new_q: f32) -> Result<()> {
        if episode_id.trim().is_empty() {
            return Ok(());
        }
        let mut connection = self
            .client
            .get_connection()
            .context("failed to open valkey connection for atomic q update")?;
        connection
            .hset::<_, _, _, ()>(&self.q_values_hash_key, episode_id, new_q)
            .with_context(|| {
                format!("failed to write atomic q update for episode `{episode_id}`")
            })?;
        Ok(())
    }

    fn update_scope_feedback_bias_atomic(&self, scope: &str, new_bias: f32) -> Result<()> {
        if scope.trim().is_empty() {
            return Ok(());
        }
        let mut connection = self
            .client
            .get_connection()
            .context("failed to open valkey connection for atomic recall-feedback update")?;
        connection
            .hset::<_, _, _, ()>(
                &self.recall_feedback_hash_key,
                scope.trim(),
                normalize_feedback_bias(new_bias),
            )
            .with_context(|| {
                format!("failed to write atomic recall-feedback update for scope `{scope}`")
            })?;
        Ok(())
    }

    fn clear_scope_feedback_bias_atomic(&self, scope: &str) -> Result<()> {
        if scope.trim().is_empty() {
            return Ok(());
        }
        let mut connection = self
            .client
            .get_connection()
            .context("failed to open valkey connection for atomic recall-feedback delete")?;
        connection
            .hdel::<_, _, ()>(&self.recall_feedback_hash_key, scope.trim())
            .with_context(|| {
                format!("failed to delete atomic recall-feedback value for scope `{scope}`")
            })?;
        Ok(())
    }
}
