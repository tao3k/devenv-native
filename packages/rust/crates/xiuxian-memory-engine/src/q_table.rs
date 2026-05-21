//! Q-table implementation for self-evolving memory.
//!
//! Implements the smoothing update
//! `Q_new = Q_old + α * (reward - Q_old)`,
//! where `α` is the learning rate and `reward` is the observed outcome.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use thiserror::Error;

use crate::EpisodeId;

/// Persistence error returned by public Q-table state APIs.
#[derive(Debug, Error)]
pub enum QTablePersistenceError {
    /// The Q-table could not be encoded as JSON.
    #[error("serialize q-table: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The Q-table file could not be read or written.
    #[error("q-table file io: {0}")]
    Io(#[from] std::io::Error),
}

/// Q-learning table for episode utility tracking.
///
/// Uses an `RwLock<HashMap<...>>` for thread-safe updates.
pub struct QTable {
    /// Internal Q-table mapping `episode_id` -> `q_value`.
    table: RwLock<HashMap<String, f32>>,
    /// Learning rate (`α`) - typically 0.1-0.3.
    learning_rate: f32,
    /// Discount factor (`γ`) - currently stored for future RL evolution.
    discount_factor: f32,
}

impl Clone for QTable {
    fn clone(&self) -> Self {
        Self {
            table: RwLock::new(HashMap::new()),
            learning_rate: self.learning_rate,
            discount_factor: self.discount_factor,
        }
    }
}

impl QTable {
    /// Create a new Q-table with default parameters.
    ///
    /// Default `learning_rate` = 0.2.
    /// Default `discount_factor` = 0.95.
    #[must_use]
    pub fn new() -> Self {
        Self::with_params(0.2, 0.95)
    }

    /// Create a new Q-table with custom parameters.
    #[must_use]
    pub fn with_params(learning_rate: f32, discount_factor: f32) -> Self {
        Self {
            table: RwLock::new(HashMap::new()),
            learning_rate,
            discount_factor,
        }
    }

    fn read_table(&self) -> RwLockReadGuard<'_, HashMap<String, f32>> {
        self.table.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write_table(&self) -> RwLockWriteGuard<'_, HashMap<String, f32>> {
        self.table.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// Update one Q-value using
    /// `Q_new = Q_old + α * (reward - Q_old)`.
    pub fn update(&self, episode_id: impl Into<EpisodeId>, reward: f32) -> f32 {
        let episode_id = episode_id.into();
        let q_old = self.get_q(episode_id.as_str());
        let q_new = q_old + self.learning_rate * (reward - q_old);
        let q_clamped = q_new.clamp(0.0, 1.0);
        self.write_table()
            .insert(episode_id.as_str().to_string(), q_clamped);
        q_clamped
    }

    /// Get the Q-value for one episode.
    ///
    /// Returns 0.5 when the episode has no stored value.
    pub fn get_q(&self, episode_id: impl Into<EpisodeId>) -> f32 {
        let episode_id = episode_id.into();
        self.read_table()
            .get(episode_id.as_str())
            .copied()
            .unwrap_or(0.5)
    }

    /// Initialize a new episode with the default Q-value.
    pub fn init_episode(&self, episode_id: impl Into<EpisodeId>) {
        let episode_id = episode_id.into();
        self.write_table()
            .entry(episode_id.as_str().to_string())
            .or_insert(0.5);
    }

    /// Get multiple Q-values at once.
    pub fn get_batch(&self, episode_ids: &[String]) -> Vec<(String, f32)> {
        let table = self.read_table();
        episode_ids
            .iter()
            .map(|id| (id.clone(), table.get(id).copied().unwrap_or(0.5)))
            .collect()
    }

    /// Batch update multiple Q-values.
    ///
    /// More efficient than individual updates when callers already have a
    /// grouped update set.
    pub fn update_batch(&self, updates: &[(String, f32)]) -> Vec<(String, f32)> {
        let mut table = self.write_table();
        updates
            .iter()
            .map(|(episode_id, reward)| {
                let q_old = table.get(episode_id).copied().unwrap_or(0.5);
                let q_new = q_old + self.learning_rate * (reward - q_old);
                let q_clamped = q_new.clamp(0.0, 1.0);
                table.insert(episode_id.clone(), q_clamped);
                (episode_id.clone(), q_clamped)
            })
            .collect()
    }

    /// Get all episode IDs in the Q-table.
    pub fn get_all_ids(&self) -> Vec<String> {
        self.read_table().keys().cloned().collect()
    }

    /// Get the number of entries in the Q-table.
    pub fn len(&self) -> usize {
        self.read_table().len()
    }

    /// Check if the Q-table is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove an entry from the Q-table.
    pub fn remove(&self, episode_id: impl Into<EpisodeId>) {
        let episode_id = episode_id.into();
        self.write_table().remove(episode_id.as_str());
    }

    /// Save the Q-table to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the table cannot be serialized or written to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), QTablePersistenceError> {
        let data = self.snapshot_map();
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(path.as_ref(), json)?;
        log::info!(
            "Saved Q-table with {} entries to {}",
            data.len(),
            path.as_ref().display()
        );
        Ok(())
    }

    /// Load the Q-table from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), QTablePersistenceError> {
        let path = path.as_ref();
        if !path.exists() {
            log::info!("No existing Q-table file at {}", path.display());
            return Ok(());
        }

        let json = std::fs::read_to_string(path)?;
        let data: HashMap<String, f32> = serde_json::from_str(&json)?;
        let count = data.len();
        *self.write_table() = data;
        log::info!("Loaded {count} Q-table entries from {}", path.display());
        Ok(())
    }

    /// Get the configured learning rate.
    pub fn learning_rate(&self) -> f32 {
        self.learning_rate
    }

    /// Get the configured discount factor.
    pub fn discount_factor(&self) -> f32 {
        self.discount_factor
    }

    /// Get a snapshot of the Q-table as a `HashMap`.
    #[must_use]
    pub fn snapshot_map(&self) -> HashMap<String, f32> {
        self.read_table().clone()
    }

    /// Replace the Q-table contents from a `HashMap`.
    pub fn replace_map(&mut self, data: HashMap<String, f32>) {
        *self.write_table() = data;
    }
}

impl Default for QTable {
    fn default() -> Self {
        Self::new()
    }
}
