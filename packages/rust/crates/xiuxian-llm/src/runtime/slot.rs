//! [AIP] Model Slot State Machine for hot-swap architecture.
//!
//! This module implements the three-state lifecycle:
//! - **Vacant**: Only metadata, no memory footprint (< 1ms)
//! - **Hibernated**: mmap established, virtual memory mapped (< 100ms)
//! - **Active**: Inference ready, weights in physical memory

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use super::executor::{ExecutorId, ModelExecutor, ModelInput, ModelOutput};
use crate::llm::error::{LlmError, LlmResult};

/// Slot state for model lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SlotState {
    /// Vacant: Only metadata loaded. No memory footprint.
    /// Transition: Vacant -> Hibernated via `hibernate()`
    /// Load time: < 1ms
    Vacant = 0,
    /// Hibernated: mmap handle established, virtual memory mapped.
    /// Physical pages loaded on-demand via page faults.
    /// Transition: Hibernated -> Active via `activate()`
    /// Load time: < 100ms (mmap only, no page faults)
    Hibernated = 1,
    /// Active: Executor loaded, ready for inference.
    /// Transition: Active -> Hibernated via `evict()`
    /// Inference ready
    Active = 2,
}

impl From<u8> for SlotState {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Hibernated,
            2 => Self::Active,
            _ => Self::Vacant,
        }
    }
}

impl std::fmt::Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "Vacant"),
            Self::Hibernated => write!(f, "Hibernated"),
            Self::Active => write!(f, "Active"),
        }
    }
}

/// Unique identifier for a model slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelSlotId(Arc<str>);

impl ModelSlotId {
    /// Creates a new model slot ID from a model root path.
    #[must_use]
    pub fn from_model_root(model_root: &str) -> Self {
        Self(Arc::from(model_root.to_string()))
    }

    /// Creates a new model slot ID from a string.
    #[must_use]
    pub fn new(id: impl Into<Self>) -> Self {
        id.into()
    }

    /// Returns the inner string reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ModelSlotId {
    fn from(id: &str) -> Self {
        Self(Arc::from(id.to_string()))
    }
}

impl From<String> for ModelSlotId {
    fn from(id: String) -> Self {
        Self(Arc::from(id))
    }
}

impl std::fmt::Display for ModelSlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about a model without loading weights.
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// Model root directory path.
    pub model_root: PathBuf,
    /// Total size of weight files in bytes.
    pub weights_size_bytes: u64,
    /// Whether a quantized snapshot (.dsq) is available.
    pub has_quantized_snapshot: bool,
    /// Model kind identifier.
    pub model_kind: &'static str,
}

/// A slot in the model bus.
///
/// This implements the three-state lifecycle:
/// Vacant -> Hibernated -> Active
pub struct ModelSlot {
    /// Slot identifier.
    id: ModelSlotId,
    /// Executor ID for this slot.
    executor_id: ExecutorId,
    /// Model metadata.
    metadata: ModelMetadata,
    /// Current state (atomic for lock-free reads).
    state: AtomicU8,
    /// The executor (set when Active).
    executor: RwLock<Option<Arc<Box<dyn ModelExecutor>>>>,
}

impl ModelSlot {
    /// Creates a new slot in Vacant state.
    #[must_use]
    pub fn vacant(id: ModelSlotId, executor_id: ExecutorId, metadata: ModelMetadata) -> Self {
        Self {
            id,
            executor_id,
            metadata,
            state: AtomicU8::new(SlotState::Vacant as u8),
            executor: RwLock::new(None),
        }
    }

    /// Returns the current state.
    pub fn state(&self) -> SlotState {
        SlotState::from(self.state.load(Ordering::Acquire))
    }

    /// Returns the slot ID.
    pub fn id(&self) -> &ModelSlotId {
        &self.id
    }

    /// Returns the executor ID.
    pub fn executor_id(&self) -> &ExecutorId {
        &self.executor_id
    }

    /// Returns the model metadata.
    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    /// Transitions from Vacant to Hibernated.
    ///
    /// This establishes mmap without loading pages into physical memory.
    /// Page faults will occur on first access.
    #[must_use]
    pub fn hibernate(&self) -> SlotState {
        let prev = self.state.compare_exchange(
            SlotState::Vacant as u8,
            SlotState::Hibernated as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        match prev {
            Ok(_) => {
                tracing::info!(
                    event = "llm.runtime.slot.hibernate",
                    slot_id = %self.id,
                    executor_id = %self.executor_id,
                    model_root = %self.metadata.model_root.display(),
                    "Model slot hibernated (Vacant -> Hibernated)"
                );
                SlotState::Vacant
            }
            Err(current) => SlotState::from(current),
        }
    }

    /// Transitions from Hibernated to Active with the given executor.
    ///
    /// This loads the executor and prepares it for inference.
    pub fn activate(&self, executor: Arc<Box<dyn ModelExecutor>>) -> SlotState {
        let memory_before = executor.memory_bytes();

        tracing::debug!(
            event = "llm.runtime.slot.activate_start",
            slot_id = %self.id,
            executor_id = %self.executor_id,
            current_state = ?self.state(),
            executor_memory_bytes = memory_before,
            "ModelSlot: Starting activation transition"
        );

        let prev = self.state.compare_exchange(
            SlotState::Hibernated as u8,
            SlotState::Active as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        match prev {
            Ok(_) => {
                tracing::debug!(
                    event = "llm.runtime.slot.activate_cas_success",
                    slot_id = %self.id,
                    "ModelSlot: CAS succeeded, storing executor and calling prewarm"
                );

                // Store the executor
                if let Ok(mut guard) = self.executor.write() {
                    *guard = Some(Arc::clone(&executor));
                }

                // Trigger prewarm
                let prewarm_start = std::time::Instant::now();
                if let Err(e) = executor.prewarm() {
                    tracing::warn!(
                        event = "llm.runtime.slot.prewarm_failed",
                        slot_id = %self.id,
                        error = %e,
                        prewarm_elapsed_ms = prewarm_start.elapsed().as_millis(),
                        "Model slot prewarm failed, inference may be slow"
                    );
                } else {
                    tracing::debug!(
                        event = "llm.runtime.slot.prewarm_success",
                        slot_id = %self.id,
                        prewarm_elapsed_ms = prewarm_start.elapsed().as_millis(),
                        "ModelSlot: Prewarm completed"
                    );
                }

                tracing::info!(
                    event = "llm.runtime.slot.activate",
                    slot_id = %self.id,
                    executor_id = %self.executor_id,
                    memory_bytes = memory_before,
                    "Model slot activated (Hibernated -> Active)"
                );
                SlotState::Hibernated
            }
            Err(current) => {
                tracing::debug!(
                    event = "llm.runtime.slot.activate_cas_failed",
                    slot_id = %self.id,
                    current_state = ?SlotState::from(current),
                    "ModelSlot: CAS failed, state already changed - EXECUTOR MAY BE LEAKED"
                );
                // Already active or other state, still store the executor
                if let Ok(mut guard) = self.executor.write() {
                    *guard = Some(executor);
                }
                SlotState::from(current)
            }
        }
    }

    /// Transitions from Active to Hibernated.
    ///
    /// This evicts the executor from active memory but keeps mmap.
    pub fn evict(&self) -> SlotState {
        tracing::debug!(
            event = "llm.runtime.slot.evict_start",
            slot_id = %self.id,
            current_state = ?self.state(),
            "ModelSlot: Starting eviction transition"
        );

        let prev = self.state.compare_exchange(
            SlotState::Active as u8,
            SlotState::Hibernated as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        match prev {
            Ok(_) => {
                // Get memory before clearing for logging
                let memory_bytes = self
                    .executor
                    .read()
                    .ok()
                    .and_then(|guard| guard.as_ref().map(|e| e.memory_bytes()))
                    .unwrap_or(0);

                // Clear the executor
                if let Ok(mut guard) = self.executor.write() {
                    *guard = None;
                }

                tracing::info!(
                    event = "llm.runtime.slot.evict",
                    slot_id = %self.id,
                    released_memory_bytes = memory_bytes,
                    "Model slot evicted (Active -> Hibernated)"
                );
                SlotState::Active
            }
            Err(current) => SlotState::from(current),
        }
    }

    /// Transitions to Vacant state.
    ///
    /// This releases all resources including mmap.
    pub fn vacate(&self) -> SlotState {
        let prev = self.state.swap(SlotState::Vacant as u8, Ordering::AcqRel);

        // Clear the executor
        if let Ok(mut guard) = self.executor.write() {
            *guard = None;
        }

        let prev_state = SlotState::from(prev);
        if prev_state != SlotState::Vacant {
            tracing::info!(
                event = "llm.runtime.slot.vacate",
                slot_id = %self.id,
                prev_state = ?prev_state,
                "Model slot vacated"
            );
        }
        prev_state
    }

    /// Returns the executor if in Active state.
    pub fn executor(&self) -> Option<Arc<Box<dyn ModelExecutor>>> {
        if self.state() == SlotState::Active {
            self.executor.read().ok().and_then(|guard| guard.clone())
        } else {
            None
        }
    }

    /// Executes inference if Active.
    ///
    /// # Errors
    ///
    /// Returns an error when the slot is not active or the executor fails.
    pub async fn execute(&self, input: ModelInput) -> LlmResult<ModelOutput> {
        let executor = self.executor().ok_or_else(|| LlmError::Internal {
            message: format!("slot {} is not active", self.id),
        })?;
        executor.execute(input).await
    }
}

/// Token for tracking active residency.
/// Used for memory pressure management.
#[derive(Debug, Clone)]
pub struct ResidencyToken {
    /// Slot ID that owns this token.
    pub slot_id: ModelSlotId,
    /// Memory bytes at activation time.
    pub memory_bytes: u64,
}

#[cfg(test)]
#[path = "../../tests/unit/runtime/slot.rs"]
mod tests;
