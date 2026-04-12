//! One-turn agent loop: user message -> LLM (+ optional tools) -> `tool_calls` -> external tool call -> repeat.

pub(crate) mod admission;
pub(crate) mod bootstrap;
mod consolidation;
mod context_budget;
mod context_budget_state;
mod embedding_runtime;
mod feedback;
mod injection;
pub(crate) mod logging;
pub(crate) mod memory;
pub(crate) mod memory_recall;
pub(crate) mod memory_recall_feedback;
pub(crate) mod memory_recall_metrics;
pub(crate) mod memory_recall_state;
mod memory_state;
pub(crate) mod memory_stream_consumer;
pub(crate) mod native_tools;
mod notification;
mod omega;
mod persistence;
pub(crate) mod reflection;
mod reflection_runtime_state;
pub(crate) mod session_context;
mod system_prompt_injection_state;
mod tool_dispatch;
mod tool_runtime_state;
pub(crate) mod tool_startup;
mod turn_execution;
mod turn_support;

pub use turn_execution::AgentStreamEvent;
pub(crate) mod zhenfa;

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
use memory_state::{MemoryStateBackend, MemoryStateLoadStatus};
use reflection::PolicyHintDirective;

const DEFAULT_MEMORY_EMBED_TIMEOUT: Duration = Duration::from_secs(3);
const DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN: Duration = Duration::from_secs(20);
const MIN_MEMORY_EMBED_TIMEOUT_MS: u64 = 100;
const MAX_MEMORY_EMBED_TIMEOUT_MS: u64 = 60_000;
const MAX_MEMORY_EMBED_COOLDOWN_MS: u64 = 300_000;

pub(crate) use admission::DownstreamAdmissionRuntimeSnapshot;
pub use consolidation::summarise_drained_turns;
pub use context_budget::prune_messages_for_token_budget;
pub use context_budget_state::{SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot};
pub use memory_recall_metrics::{MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot};
pub use memory_recall_state::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};
pub use memory_state::MemoryRuntimeStatusSnapshot;
pub use native_tools::registry::NativeToolRegistry;
pub use notification::{NotificationDispatcher, NotificationProvider};
pub use session_context::{
    SessionContextMode, SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
};
pub use system_prompt_injection_state::SessionSystemPromptInjectionSnapshot;

/// Explicit session-level recall feedback direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRecallFeedbackDirection {
    Up,
    Down,
}

/// Result of applying explicit session-level recall feedback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionRecallFeedbackUpdate {
    pub previous_bias: f32,
    pub updated_bias: f32,
    pub direction: SessionRecallFeedbackDirection,
}

/// Agent: config + session store (or bounded session) + LLM client + optional external tool pool + optional memory.
pub struct Agent {
    config: AgentConfig,
    session: SessionStore,
    /// Idle timeout before session context is auto-reset.
    session_reset_idle_timeout_ms: Option<u64>,
    /// Last-activity timestamp by logical session id.
    session_last_activity_unix_ms: Arc<RwLock<HashMap<String, u64>>>,
    /// When set, session history is bounded; context built from recent turns.
    bounded_session: Option<BoundedSessionStore>,
    /// When set (and window enabled), consolidation stores episodes into omni-memory.
    memory_store: Option<Arc<EpisodeStore>>,
    /// Memory persistence backend for episode/Q state snapshots.
    memory_state_backend: Option<Arc<MemoryStateBackend>>,
    /// Startup load status for memory state persistence.
    memory_state_load_status: MemoryStateLoadStatus,
    /// Embedding client for semantic memory recall/store.
    embedding_client: Option<EmbeddingClient>,
    /// Stateful timeout/cooldown guard for memory embedding requests.
    embedding_runtime: Option<Arc<xiuxian_llm::embedding::runtime::EmbeddingRuntime>>,
    /// Most recent context-budget report by logical session id.
    context_budget_snapshots: Arc<RwLock<HashMap<String, SessionContextBudgetSnapshot>>>,
    /// Process-level memory recall metrics snapshot (for diagnostics dashboards).
    memory_recall_metrics: Arc<RwLock<memory_recall_metrics::MemoryRecallMetricsState>>,
    /// Session-level recall feedback bias (-1: broaden recall, +1: tighten recall).
    memory_recall_feedback: Arc<RwLock<HashMap<String, f32>>>,
    /// Session-level injected system prompt window (XML Q&A).
    system_prompt_injection: Arc<RwLock<HashMap<String, SessionSystemPromptInjectionSnapshot>>>,
    /// One-shot next-turn policy hints derived from reflection lifecycle.
    reflection_policy_hints: Arc<RwLock<HashMap<String, PolicyHintDirective>>>,
    /// Counter used by periodic memory decay policy.
    memory_decay_turn_counter: Arc<AtomicU64>,
    /// Native in-process tool registry.
    native_tools: Arc<NativeToolRegistry>,
    /// Optional Zhixing-Heyi runtime mounted into the agent.
    heyi: Option<Arc<ZhixingHeyi>>,
    /// Optional in-process Zhenfa tool bridge.
    zhenfa_tools: Option<Arc<crate::agent::zhenfa::ZhenfaToolBridge>>,
    /// Downstream saturation admission policy.
    downstream_admission_policy: admission::DownstreamAdmissionPolicy,
    /// Downstream saturation admission metrics.
    downstream_admission_metrics: admission::DownstreamAdmissionMetrics,
    llm: LlmClient,
    tool_runtime: Option<crate::ToolClientPool>,
    memory_stream_consumer_task: Option<tokio::task::JoinHandle<()>>,
    _hot_reload_driver: Option<HotReloadDriver>,
    service_mount_records: Arc<RwLock<Vec<crate::agent::bootstrap::ServiceMountRecord>>>,
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
