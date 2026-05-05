//! Core agent state and lifecycle hooks.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use tokio::sync::RwLock;
use xiuxian_memory_engine::EpisodeStore;
use xiuxian_qianhuan::HotReloadDriver;
use xiuxian_zhixing::ZhixingHeyi;

use crate::config::AgentConfig;
use crate::embedding::EmbeddingClient;
use crate::llm::LlmClient;
use crate::session::{BoundedSessionStore, SessionStore};

use super::memory_state::{MemoryStateBackend, MemoryStateLoadStatus};
use super::reflection::PolicyHintDirective;
use super::{
    NativeToolRegistry, SessionContextBudgetSnapshot, SessionSystemPromptInjectionSnapshot,
    admission, memory_recall_metrics,
};

pub(crate) const DEFAULT_MEMORY_EMBED_TIMEOUT: Duration = Duration::from_secs(3);
pub(crate) const DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN: Duration = Duration::from_secs(20);
pub(crate) const MIN_MEMORY_EMBED_TIMEOUT_MS: u64 = 100;
pub(crate) const MAX_MEMORY_EMBED_TIMEOUT_MS: u64 = 60_000;
pub(crate) const MAX_MEMORY_EMBED_COOLDOWN_MS: u64 = 300_000;

/// Agent: config + session store (or bounded session) + LLM client + optional external tool pool + optional memory.
pub struct Agent {
    pub(crate) config: AgentConfig,
    pub(crate) session: SessionStore,
    /// Idle timeout before session context is auto-reset.
    pub(crate) session_reset_idle_timeout_ms: Option<u64>,
    /// Last-activity timestamp by logical session id.
    pub(crate) session_last_activity_unix_ms: Arc<RwLock<HashMap<String, u64>>>,
    /// When set, session history is bounded; context built from recent turns.
    pub(crate) bounded_session: Option<BoundedSessionStore>,
    /// When set (and window enabled), consolidation stores episodes into omni-memory.
    pub(crate) memory_store: Option<Arc<EpisodeStore>>,
    /// Memory persistence backend for episode/Q state snapshots.
    pub(crate) memory_state_backend: Option<Arc<MemoryStateBackend>>,
    /// Startup load status for memory state persistence.
    pub(in crate::agent) memory_state_load_status: MemoryStateLoadStatus,
    /// Embedding client for semantic memory recall/store.
    pub(crate) embedding_client: Option<EmbeddingClient>,
    /// Stateful timeout/cooldown guard for memory embedding requests.
    pub(crate) embedding_runtime: Option<Arc<xiuxian_llm::embedding::EmbeddingRuntime>>,
    /// Most recent context-budget report by logical session id.
    pub(crate) context_budget_snapshots: Arc<RwLock<HashMap<String, SessionContextBudgetSnapshot>>>,
    /// Process-level memory recall metrics snapshot (for diagnostics dashboards).
    pub(crate) memory_recall_metrics: Arc<RwLock<memory_recall_metrics::MemoryRecallMetricsState>>,
    /// Session-level recall feedback bias (-1: broaden recall, +1: tighten recall).
    pub(crate) memory_recall_feedback: Arc<RwLock<HashMap<String, f32>>>,
    /// Session-level injected system prompt window (XML Q&A).
    pub(crate) system_prompt_injection:
        Arc<RwLock<HashMap<String, SessionSystemPromptInjectionSnapshot>>>,
    /// One-shot next-turn policy hints derived from reflection lifecycle.
    pub(crate) reflection_policy_hints: Arc<RwLock<HashMap<String, PolicyHintDirective>>>,
    /// Counter used by periodic memory decay policy.
    pub(crate) memory_decay_turn_counter: Arc<AtomicU64>,
    /// Native in-process tool registry.
    pub(crate) native_tools: Arc<NativeToolRegistry>,
    /// Optional Zhixing-Heyi runtime mounted into the agent.
    pub(crate) heyi: Option<Arc<ZhixingHeyi>>,
    /// Optional in-process Zhenfa tool bridge.
    pub(crate) zhenfa_tools: Option<Arc<crate::agent::zhenfa::ZhenfaToolBridge>>,
    /// Downstream saturation admission policy.
    pub(crate) downstream_admission_policy: admission::DownstreamAdmissionPolicy,
    /// Downstream saturation admission metrics.
    pub(crate) downstream_admission_metrics: admission::DownstreamAdmissionMetrics,
    pub(crate) llm: LlmClient,
    pub(crate) tool_runtime: Option<crate::ToolClientPool>,
    pub(crate) memory_stream_consumer_task: Option<tokio::task::JoinHandle<()>>,
    pub(crate) _hot_reload_driver: Option<HotReloadDriver>,
    pub(crate) service_mount_records: Arc<RwLock<Vec<crate::agent::bootstrap::ServiceMountRecord>>>,
}

impl Drop for Agent {
    fn drop(&mut self) {
        if let Some(task) = self.memory_stream_consumer_task.take() {
            task.abort();
        }
    }
}

impl Agent {
    #[must_use]
    /// Returns the mounted Zhixing-Heyi runtime when the agent was bootstrapped
    /// with one.
    pub fn get_heyi(&self) -> Option<Arc<ZhixingHeyi>> {
        self.heyi.as_ref().map(Arc::clone)
    }
}
