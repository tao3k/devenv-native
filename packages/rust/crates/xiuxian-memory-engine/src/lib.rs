//! xiuxian-memory-engine - Self-evolving memory engine with Q-Learning and Two-Phase Search.
//!
//! Provides high-performance memory management for AI agents:
//! - Episode storage with vector similarity search
//! - Q-Learning for utility-based episode selection
//! - Two-phase search (semantic recall + Q-value reranking)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Host Layer (Orchestration)               │
//! │  - Workflow orchestration                                  │
//! │  - State management                                        │
//! │  - LLM interaction                                         │
//! └─────────────────────────────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Rust Layer (Performance Core)           │
//! │  - Episode Store (episodic state backend)                 │
//! │  - Q-Table (Q-Learning)                                   │
//! │  - Two-Phase Search                                       │
//! │  - Intent Encoding                                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Namespace
//!
//! ```rust
//! use xiuxian_memory_engine::{Episode, QTable, EpisodeStore, TwoPhaseSearch};
//! ```
//!
//! # Examples
//!
//! ```rust
//! use xiuxian_memory_engine::{Episode, EpisodeStore, StoreConfig};
//!
//! let config = StoreConfig {
//!     path: "memory".to_string(),
//!     embedding_dim: 384,
//!     table_name: "episodes".to_string(),
//! };
//! let store = EpisodeStore::new(config);
//! ```
//!
//! ```rust
//! use xiuxian_memory_engine::{QTable, calculate_score};
//!
//! let q_table = QTable::new();
//! q_table.update("ep-001", 1.0);  // Update with reward
//! let q_value = q_table.get_q("ep-001");
//! ```

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = rust_project_harness_gate::memory_engine_rust_harness_config()
);

// ============================================================================
// Core modules
// ============================================================================

mod encoder;
mod episode;
mod gate;
mod persistence;
mod projection;
mod q_table;
mod recall_feedback;
mod schema;
mod state_backend;
mod store;
mod two_phase;

// ============================================================================
// Public exports
// ============================================================================

pub use encoder::IntentEncoder;
pub use episode::{Episode, EpisodeDraft, EpisodeId};
pub use gate::{
    MemoryGateDecision, MemoryGateEvent, MemoryGateEventInput, MemoryGateMemoryId,
    MemoryGatePolicy, MemoryGateSessionId, MemoryGateTurnId, MemoryGateVerdict,
    MemoryLifecycleState, MemoryPromotionTarget, MemoryUtilityLedger,
};
pub use projection::{MemoryProjectionFilter, MemoryProjectionRow, MemoryProjectionTimestampMs};
pub use q_table::{QTable, QTablePersistenceError};
pub use recall_feedback::{
    RecallFeedbackOutcome, RecallPlanTuning, apply_feedback_to_plan_tuning,
    normalize_feedback_bias, update_feedback_bias,
};
pub use schema::EpisodeMetadata;
#[cfg(feature = "valkey")]
pub use state_backend::ValkeyMemoryStateStore;
pub use state_backend::{
    LocalMemoryStateStore, MemoryStateStore, ValkeyStateHashKeys,
    default_valkey_recall_feedback_hash_key, default_valkey_state_hash_keys,
    default_valkey_state_key,
};
pub use store::{
    EpisodeStore, MemoryStateSnapshot, ScopedTwoPhaseEmbeddingRecallRequest,
    ScopedTwoPhaseRecallRequest, StoreConfig,
};
pub use two_phase::{TwoPhaseConfig, TwoPhaseSearch, TwoPhaseSearchRequest, calculate_score};
