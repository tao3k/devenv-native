use anyhow::{Result, bail};
use xiuxian_macros::env_non_empty;
use xiuxian_memory_engine::{
    EpisodeStore, LocalMemoryStateStore, MemoryStateStore, StoreConfig, ValkeyMemoryStateStore,
    default_valkey_state_key,
};

use super::Agent;
use crate::config::MemoryConfig;
use crate::env_parse::{parse_bool_from_env, resolve_valkey_url_env};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistenceBackendMode {
    Auto,
    Local,
    Valkey,
}

pub(crate) enum MemoryStateBackend {
    Local(LocalMemoryStateStore),
    Valkey(Box<ValkeyMemoryStateStore>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MemoryStateLoadStatus {
    NotConfigured,
    Loaded,
    LoadFailedContinue,
}

impl MemoryStateLoadStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::Loaded => "loaded",
            Self::LoadFailedContinue => "load_failed_continue",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuntimeStatusSnapshot {
    pub enabled: bool,
    pub configured_backend: Option<String>,
    pub active_backend: Option<&'static str>,
    pub strict_startup: Option<bool>,
    pub startup_load_status: &'static str,
    pub store_path: Option<String>,
    pub table_name: Option<String>,
    pub gate_promote_threshold: Option<f32>,
    pub gate_obsolete_threshold: Option<f32>,
    pub gate_promote_min_usage: Option<u32>,
    pub gate_obsolete_min_usage: Option<u32>,
    pub gate_promote_failure_rate_ceiling: Option<f32>,
    pub gate_obsolete_failure_rate_floor: Option<f32>,
    pub gate_promote_min_ttl_score: Option<f32>,
    pub gate_obsolete_max_ttl_score: Option<f32>,
    pub episodes_total: Option<usize>,
    pub q_values_total: Option<usize>,
}

impl MemoryStateBackend {
    pub(super) fn from_config(memory_cfg: &MemoryConfig) -> Result<Self> {
        let mode = resolve_mode(&memory_cfg.persistence_backend)?;
        let redis_url = non_empty_string(memory_cfg.persistence_valkey_url.clone())
            .or_else(resolve_valkey_url_env);
        let strict_startup_override =
            parse_bool_from_env("XIUXIAN_DAOCHANG_MEMORY_PERSISTENCE_STRICT_STARTUP")
                .or(memory_cfg.persistence_strict_startup);
        let key_prefix = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_VALKEY_KEY_PREFIX")
            .or_else(|| non_empty_string(Some(memory_cfg.persistence_key_prefix.clone())))
            .unwrap_or_else(|| "xiuxian-daochang:memory".to_string());

        let store_config = StoreConfig {
            path: memory_cfg.path.clone(),
            embedding_dim: memory_cfg.embedding_dim,
            table_name: memory_cfg.table_name.clone(),
        };

        match mode {
            PersistenceBackendMode::Local => Ok(Self::Local(LocalMemoryStateStore::new())),
            PersistenceBackendMode::Valkey => {
                let redis_url = redis_url.ok_or_else(|| {
                    anyhow::anyhow!(
                        "memory persistence backend=valkey requires valkey url (session.valkey_url, XIUXIAN_WENDAO_VALKEY_URL, or VALKEY_URL)"
                    )
                })?;
                let key = default_valkey_state_key(&key_prefix, &store_config);
                let strict_startup = strict_startup_override.unwrap_or(true);
                Ok(Self::Valkey(Box::new(ValkeyMemoryStateStore::new(
                    redis_url,
                    key,
                    strict_startup,
                )?)))
            }
            PersistenceBackendMode::Auto => {
                if let Some(redis_url) = redis_url {
                    let key = default_valkey_state_key(&key_prefix, &store_config);
                    let strict_startup = strict_startup_override.unwrap_or(true);
                    Ok(Self::Valkey(Box::new(ValkeyMemoryStateStore::new(
                        redis_url,
                        key,
                        strict_startup,
                    )?)))
                } else {
                    Ok(Self::Local(LocalMemoryStateStore::new()))
                }
            }
        }
    }

    fn as_store(&self) -> &dyn MemoryStateStore {
        match self {
            Self::Local(store) => store,
            Self::Valkey(store) => store.as_ref(),
        }
    }

    pub(super) fn backend_name(&self) -> &'static str {
        self.as_store().backend_name()
    }

    pub(super) fn strict_startup(&self) -> bool {
        self.as_store().strict_startup()
    }

    pub(super) fn load(&self, store: &mut EpisodeStore) -> Result<()> {
        self.as_store().load(store)
    }

    pub(super) fn save(&self, store: &EpisodeStore) -> Result<()> {
        self.as_store().save(store)
    }

    pub(super) fn update_q_atomic(&self, episode_id: &str, new_q: f32) -> Result<()> {
        self.as_store().update_q_atomic(episode_id, new_q)
    }

    pub(super) fn update_scope_feedback_bias_atomic(
        &self,
        scope: &str,
        new_bias: f32,
    ) -> Result<()> {
        self.as_store()
            .update_scope_feedback_bias_atomic(scope, new_bias)
    }

    pub(super) fn clear_scope_feedback_bias_atomic(&self, scope: &str) -> Result<()> {
        self.as_store().clear_scope_feedback_bias_atomic(scope)
    }
}

fn resolve_mode(raw: &str) -> Result<PersistenceBackendMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(PersistenceBackendMode::Auto),
        "local" => Ok(PersistenceBackendMode::Local),
        "valkey" => Ok(PersistenceBackendMode::Valkey),
        other => bail!("invalid memory persistence backend `{other}`; expected auto|local|valkey"),
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    env_non_empty!(name)
}

fn non_empty_string(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty())
}

impl Agent {
    pub(in crate::agent) fn persist_memory_q_atomic(
        &self,
        session_id: Option<&str>,
        episode_id: &str,
        q_value: f32,
        reason: &str,
    ) {
        let Some(backend) = self.memory_state_backend.as_ref() else {
            return;
        };
        let session_id = session_id.unwrap_or_default();
        match backend.update_q_atomic(episode_id, q_value) {
            Ok(()) => {
                tracing::debug!(
                    event = "agent.memory.state.q_atomic_persisted",
                    backend = backend.backend_name(),
                    session_id,
                    reason,
                    episode_id,
                    q_value,
                    "memory q-value persisted with atomic backend update"
                );
            }
            Err(error) => {
                tracing::warn!(
                    event = "agent.memory.state.q_atomic_persist_failed",
                    backend = backend.backend_name(),
                    session_id,
                    reason,
                    episode_id,
                    q_value,
                    error = %error,
                    "failed to persist memory q-value atomically"
                );
            }
        }
    }

    pub(in crate::agent) fn persist_memory_recall_feedback_bias_atomic(
        &self,
        session_id: &str,
        bias: f32,
        reason: &str,
    ) {
        let Some(backend) = self.memory_state_backend.as_ref() else {
            return;
        };
        match backend.update_scope_feedback_bias_atomic(session_id, bias) {
            Ok(()) => {
                tracing::debug!(
                    event = "agent.memory.state.recall_feedback_atomic_persisted",
                    backend = backend.backend_name(),
                    session_id,
                    reason,
                    bias,
                    "memory recall-feedback bias persisted with atomic backend update"
                );
            }
            Err(error) => {
                tracing::warn!(
                    event = "agent.memory.state.recall_feedback_atomic_persist_failed",
                    backend = backend.backend_name(),
                    session_id,
                    reason,
                    bias,
                    error = %error,
                    "failed to persist memory recall-feedback bias atomically"
                );
            }
        }
    }

    pub(in crate::agent) fn clear_memory_recall_feedback_bias_atomic(
        &self,
        session_id: &str,
        reason: &str,
    ) {
        let Some(backend) = self.memory_state_backend.as_ref() else {
            return;
        };
        match backend.clear_scope_feedback_bias_atomic(session_id) {
            Ok(()) => {
                tracing::debug!(
                    event = "agent.memory.state.recall_feedback_atomic_cleared",
                    backend = backend.backend_name(),
                    session_id,
                    reason,
                    "memory recall-feedback bias cleared with atomic backend delete"
                );
            }
            Err(error) => {
                tracing::warn!(
                    event = "agent.memory.state.recall_feedback_atomic_clear_failed",
                    backend = backend.backend_name(),
                    session_id,
                    reason,
                    error = %error,
                    "failed to clear memory recall-feedback bias atomically"
                );
            }
        }
    }

    /// Return a point-in-time snapshot of memory runtime configuration and health.
    #[must_use]
    pub fn inspect_memory_runtime_status(&self) -> MemoryRuntimeStatusSnapshot {
        let (episodes_total, q_values_total) =
            self.memory_store.as_ref().map_or((None, None), |store| {
                let stats = store.stats();
                (Some(stats.total_episodes), Some(stats.q_table_size))
            });

        MemoryRuntimeStatusSnapshot {
            enabled: self.config.memory.is_some(),
            configured_backend: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.persistence_backend.clone()),
            active_backend: self
                .memory_state_backend
                .as_ref()
                .map(|backend| backend.backend_name()),
            strict_startup: self
                .memory_state_backend
                .as_ref()
                .map(|backend| backend.strict_startup()),
            startup_load_status: self.memory_state_load_status.as_str(),
            store_path: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.path.clone()),
            table_name: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.table_name.clone()),
            gate_promote_threshold: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_promote_threshold),
            gate_obsolete_threshold: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_obsolete_threshold),
            gate_promote_min_usage: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_promote_min_usage),
            gate_obsolete_min_usage: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_obsolete_min_usage),
            gate_promote_failure_rate_ceiling: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_promote_failure_rate_ceiling),
            gate_obsolete_failure_rate_floor: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_obsolete_failure_rate_floor),
            gate_promote_min_ttl_score: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_promote_min_ttl_score),
            gate_obsolete_max_ttl_score: self
                .config
                .memory
                .as_ref()
                .map(|memory_cfg| memory_cfg.gate_obsolete_max_ttl_score),
            episodes_total,
            q_values_total,
        }
    }
}
