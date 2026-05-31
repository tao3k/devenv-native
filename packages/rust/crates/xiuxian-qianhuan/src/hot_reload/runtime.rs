//! Hot-reload runtime coordination for registered Qianhuan targets.

use super::{HotReloadTarget, HotReloadTargetId, HotReloadVersionBackend};
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Trigger source for one hot-reload attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HotReloadTrigger {
    /// Triggered by a local file-system change.
    LocalPathChange {
        /// Path that triggered the local reload flow.
        path: PathBuf,
    },
    /// Triggered by a remote version bump from a shared backend.
    RemoteVersionSync,
}

/// Result status for one hot-reload target execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HotReloadStatus {
    /// Target was reloaded and state changed.
    Reloaded,
    /// Reload callback executed but no state change was detected.
    NoChange,
    /// Reload callback failed.
    Failed,
}

/// One hot-reload execution outcome entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HotReloadOutcome {
    /// Target identifier.
    pub target_id: String,
    /// Trigger source.
    pub trigger: HotReloadTrigger,
    /// Result status.
    pub status: HotReloadStatus,
    /// Local version after this execution.
    pub version: u64,
    /// Optional error detail for `Failed` status.
    pub error: Option<String>,
}

/// Shared runtime that coordinates registered hot-reload targets.
pub struct HotReloadRuntime {
    backend: Option<Arc<dyn HotReloadVersionBackend>>,
    state: RwLock<BTreeMap<String, RegisteredTarget>>,
}

impl HotReloadRuntime {
    /// Creates a runtime with an optional shared version backend.
    #[must_use]
    pub fn new(backend: Option<Arc<dyn HotReloadVersionBackend>>) -> Self {
        Self {
            backend,
            state: RwLock::new(BTreeMap::new()),
        }
    }

    /// Registers one reload target.
    ///
    /// # Errors
    ///
    /// Returns an error when the state lock is poisoned.
    pub fn register_target(&self, target: HotReloadTarget) -> Result<()> {
        let target_id = target.id().to_string();
        let backend_target_id = HotReloadTargetId::new(target.id());
        let initial_version = match self.backend.as_ref() {
            Some(backend) => backend.read_version(&backend_target_id)?.unwrap_or(0),
            None => 0,
        };

        let mut guard = self
            .state
            .write()
            .map_err(|_| anyhow!("hot reload runtime lock poisoned"))?;
        guard.insert(
            target_id,
            RegisteredTarget {
                target: Arc::new(target),
                local_version: initial_version,
            },
        );
        Ok(())
    }

    /// Handles one local file change path and runs matched targets.
    ///
    /// # Errors
    ///
    /// Returns an error when runtime state locks are poisoned or when the
    /// shared backend is unavailable.
    pub fn on_local_path_change(&self, path: &Path) -> Result<Vec<HotReloadOutcome>> {
        let matched = {
            let guard = self
                .state
                .read()
                .map_err(|_| anyhow!("hot reload runtime lock poisoned"))?;
            guard
                .iter()
                .filter_map(|(target_id, state)| {
                    if state.target.matches_path(path) {
                        Some((
                            target_id.clone(),
                            Arc::clone(&state.target),
                            state.local_version,
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        let mut outcomes = Vec::new();
        for (target_id, target, local_version) in matched {
            let trigger = HotReloadTrigger::LocalPathChange {
                path: path.to_path_buf(),
            };
            let callback_result = target.reload_if_changed();
            let (status, error) = match callback_result {
                Ok(true) => (HotReloadStatus::Reloaded, None),
                Ok(false) => (HotReloadStatus::NoChange, None),
                Err(error) => (HotReloadStatus::Failed, Some(error.to_string())),
            };
            let mut version = local_version;
            if status == HotReloadStatus::Reloaded {
                version = self.bump_version_and_persist(&target_id, local_version)?;
            }
            self.update_local_version(&target_id, version)?;
            outcomes.push(HotReloadOutcome {
                target_id,
                trigger,
                status,
                version,
                error,
            });
        }

        Ok(outcomes)
    }

    /// Synchronizes local targets from remote backend versions.
    ///
    /// # Errors
    ///
    /// Returns an error when runtime state locks are poisoned or when backend
    /// operations fail.
    pub fn sync_remote_versions(&self) -> Result<Vec<HotReloadOutcome>> {
        let Some(backend) = self.backend.as_ref() else {
            return Ok(Vec::new());
        };

        let snapshot = {
            let guard = self
                .state
                .read()
                .map_err(|_| anyhow!("hot reload runtime lock poisoned"))?;
            guard
                .iter()
                .map(|(target_id, state)| {
                    (
                        target_id.clone(),
                        Arc::clone(&state.target),
                        state.local_version,
                    )
                })
                .collect::<Vec<_>>()
        };

        let mut outcomes = Vec::new();
        for (target_id, target, local_version) in snapshot {
            let backend_target_id = HotReloadTargetId::new(target_id.as_str());
            let remote_version = backend.read_version(&backend_target_id)?.unwrap_or(0);
            if remote_version <= local_version {
                continue;
            }
            let callback_result = target.reload_if_changed();
            let (status, error) = match callback_result {
                Ok(true) => (HotReloadStatus::Reloaded, None),
                Ok(false) => (HotReloadStatus::NoChange, None),
                Err(error) => (HotReloadStatus::Failed, Some(error.to_string())),
            };
            self.update_local_version(&target_id, remote_version)?;
            outcomes.push(HotReloadOutcome {
                target_id,
                trigger: HotReloadTrigger::RemoteVersionSync,
                status,
                version: remote_version,
                error,
            });
        }

        Ok(outcomes)
    }

    /// Returns local version snapshot for observability/testing.
    ///
    /// # Errors
    ///
    /// Returns an error when runtime state lock is poisoned.
    pub fn local_versions(&self) -> Result<BTreeMap<String, u64>> {
        let guard = self
            .state
            .read()
            .map_err(|_| anyhow!("hot reload runtime lock poisoned"))?;
        Ok(guard
            .iter()
            .map(|(target_id, state)| (target_id.clone(), state.local_version))
            .collect())
    }

    /// Collects watcher roots and include patterns for all registered targets.
    ///
    /// # Errors
    ///
    /// Returns an error when runtime state lock is poisoned.
    pub fn watcher_roots_and_patterns(&self) -> Result<(Vec<PathBuf>, Vec<String>)> {
        let guard = self
            .state
            .read()
            .map_err(|_| anyhow!("hot reload runtime lock poisoned"))?;
        let mut roots = Vec::new();
        let mut patterns = Vec::new();
        for state in guard.values() {
            roots.extend(state.target.roots().iter().cloned());
            patterns.extend(state.target.include_globs().iter().cloned());
        }
        roots.sort();
        roots.dedup();
        patterns.sort();
        patterns.dedup();
        Ok((roots, patterns))
    }

    fn bump_version_and_persist(&self, target_id: &str, current: u64) -> Result<u64> {
        if let Some(backend) = self.backend.as_ref() {
            let backend_target_id = HotReloadTargetId::new(target_id);
            return backend.bump_version(&backend_target_id);
        }
        Ok(current.saturating_add(1))
    }

    fn update_local_version(&self, target_id: &str, version: u64) -> Result<()> {
        let mut guard = self
            .state
            .write()
            .map_err(|_| anyhow!("hot reload runtime lock poisoned"))?;
        let Some(state) = guard.get_mut(target_id) else {
            return Err(anyhow!("hot reload target not found: {target_id}"));
        };
        state.local_version = version;
        Ok(())
    }
}

struct RegisteredTarget {
    target: Arc<HotReloadTarget>,
    local_version: u64,
}
