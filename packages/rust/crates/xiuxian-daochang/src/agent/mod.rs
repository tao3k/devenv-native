//! One-turn agent loop: user message -> LLM (+ optional tools) -> `tool_calls` -> external tool call -> repeat.

pub(crate) mod admission;
pub(crate) mod bootstrap;
mod consolidation;
mod context_budget;
mod context_budget_state;
mod core;
mod embedding_runtime;
mod feedback;
mod feedback_types;
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
pub(crate) mod zhenfa;

pub(crate) use admission::DownstreamAdmissionRuntimeSnapshot;
pub use bootstrap::{ServiceMountCategory, ServiceMountRecord};
pub use consolidation::{DrainedTurn, DrainedTurnSummary, summarise_drained_turns};
pub(crate) use consolidation::{build_consolidated_summary_text, now_unix_ms};
pub use context_budget::prune_messages_for_token_budget;
pub use context_budget_state::{SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot};
pub use core::Agent;
pub(crate) use core::{
    DEFAULT_MEMORY_EMBED_TIMEOUT, DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN,
    MAX_MEMORY_EMBED_COOLDOWN_MS, MAX_MEMORY_EMBED_TIMEOUT_MS, MIN_MEMORY_EMBED_TIMEOUT_MS,
};
pub use feedback_types::{SessionRecallFeedbackDirection, SessionRecallFeedbackUpdate};
pub use memory_recall_metrics::{MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot};
pub use memory_recall_state::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};
pub use memory_state::MemoryRuntimeStatusSnapshot;
pub use native_tools::registry::NativeToolRegistry;
pub use notification::{NotificationDispatcher, NotificationProvider};
pub use session_context::{
    SessionContextMode, SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
};
pub use system_prompt_injection_state::SessionSystemPromptInjectionSnapshot;
