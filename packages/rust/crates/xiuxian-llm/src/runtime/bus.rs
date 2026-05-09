//! [AIP] Omni `ModelBus` - Global Model Registry and Resource Reclamation.
//!
//! This module implements the central registry for all model executors,
//! supporting hot-swap, memory pressure management. and prewarm capabilities.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::executor::{ExecutorId, ModelExecutor, ModelInput, ModelOutput};
use super::slot::{ModelMetadata, ModelSlot, ModelSlotId, SlotState};
use crate::llm::error::{LlmError, LlmResult};

/// Global model bus for hot-swap model management.
///
/// This registry manages all model slots and handles:
/// - Registration of model slots
/// - State transitions (Vacant -> Hibernated -> Active)
/// - Memory pressure management
/// - Prewarm for fast cold starts
pub struct ModelBus {
    /// All registered slots.
    slots: Mutex<HashMap<ModelSlotId, Arc<ModelSlot>>>,
    /// Executor factories.
    factories: Mutex<HashMap<ExecutorId, Arc<dyn ExecutorFactory>>>,
}

impl ModelBus {
    /// Creates a new model bus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: Mutex::new(HashMap::new()),
            factories: Mutex::new(HashMap::new()),
        }
    }

    /// Registers an executor factory.
    pub fn register_factory(&self, factory: Arc<dyn ExecutorFactory>) {
        if let Ok(mut guard) = self.factories.lock() {
            guard.insert(factory.id().clone(), factory);
        }
    }

    /// Registers a model slot.
    pub fn register(&self, slot: ModelSlot) -> Arc<ModelSlot> {
        let id = slot.id().clone();
        let arc_slot = Arc::new(slot);
        if let Ok(mut guard) = self.slots.lock() {
            guard.insert(id, Arc::clone(&arc_slot));
        }
        arc_slot
    }

    /// Returns a registered slot by ID.
    pub fn get(&self, id: &ModelSlotId) -> Option<Arc<ModelSlot>> {
        self.slots
            .lock()
            .ok()
            .and_then(|guard| guard.get(id).cloned())
    }

    /// Hibernates a slot (Vacant -> Hibernated).
    ///
    /// This establishes mmap without loading pages.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested model slot is not registered.
    pub fn hibernate(&self, id: &ModelSlotId) -> LlmResult<SlotState> {
        let slot = self.get(id).ok_or_else(|| LlmError::Internal {
            message: format!("model slot not found: {id}"),
        })?;
        Ok(slot.hibernate())
    }

    /// Activates a slot (Hibernated -> Active).
    ///
    /// This loads the executor and prepares for inference.
    ///
    /// # Errors
    ///
    /// Returns an error when the slot or its executor factory cannot be found,
    /// or when executor creation fails.
    ///
    /// # Memory Safety
    ///
    /// This method is idempotent - if the slot is already Active with an executor,
    /// it returns immediately without creating a new executor (preventing memory leaks).
    pub fn activate(&self, id: &ModelSlotId) -> LlmResult<SlotState> {
        let slot = self.get(id).ok_or_else(|| LlmError::Internal {
            message: format!("model slot not found: {id}"),
        })?;

        // CRITICAL: Check if already active to prevent memory explosion
        // from repeated executor creation
        if slot.state() == SlotState::Active && slot.executor().is_some() {
            tracing::debug!(
                event = "llm.runtime.bus.activate_skip",
                slot_id = %id,
                "ModelBus: Slot already active with executor, skipping creation"
            );
            return Ok(SlotState::Active);
        }

        tracing::debug!(
            event = "llm.runtime.bus.activate_start",
            slot_id = %id,
            current_state = ?slot.state(),
            executor_id = %slot.executor_id(),
            "ModelBus: Starting slot activation"
        );

        // Get factory and create executor
        let factory = {
            let guard = self.factories.lock().map_err(|_| LlmError::Internal {
                message: "factory registry mutex poisoned".to_string(),
            })?;
            guard
                .get(slot.executor_id())
                .cloned()
                .ok_or_else(|| LlmError::Internal {
                    message: format!("executor factory not found: {}", slot.executor_id()),
                })?
        };

        tracing::debug!(
            event = "llm.runtime.bus.factory_create_start",
            slot_id = %id,
            factory_id = %factory.id(),
            "ModelBus: Calling factory.create() - MEMORY ALLOCATION POINT"
        );

        let executor = factory.create(slot.metadata())?;

        tracing::debug!(
            event = "llm.runtime.bus.factory_create_done",
            slot_id = %id,
            executor_id = %executor.id(),
            memory_bytes = executor.memory_bytes(),
            "ModelBus: Executor created, calling slot.activate()"
        );

        Ok(slot.activate(executor))
    }

    /// Evicts a slot (Active -> Hibernated).
    pub fn evict(&self, id: &ModelSlotId) -> SlotState {
        if let Some(slot) = self.get(id) {
            slot.evict()
        } else {
            SlotState::Vacant
        }
    }

    /// Vacates a slot (releases all resources).
    pub fn vacate(&self, id: &ModelSlotId) -> SlotState {
        if let Some(slot) = self.get(id) {
            slot.vacate()
        } else {
            SlotState::Vacant
        }
    }

    /// Returns snapshot of all slots and their states.
    pub fn snapshot(&self) -> Vec<(ModelSlotId, SlotState)> {
        self.slots
            .lock()
            .map(|guard| {
                guard
                    .iter()
                    .map(|(id, slot)| (id.clone(), slot.state()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the total memory footprint of Active slots.
    pub fn active_memory_bytes(&self) -> u64 {
        self.slots.lock().map_or(0, |guard| {
            guard
                .values()
                .filter_map(|slot| {
                    if slot.state() == SlotState::Active {
                        slot.executor().map(|e| e.memory_bytes())
                    } else {
                        None
                    }
                })
                .sum()
        })
    }

    /// Prewarms all hibernated slots.
    ///
    /// This transitions Vacant slots to Hibernated and prepares them
    /// for fast activation on first inference.
    ///
    /// # Errors
    ///
    /// Returns an error when one of the slots cannot be prepared for
    /// hibernation, or when the slot registry mutex is poisoned.
    pub fn prewarm_all(&self) -> LlmResult<usize> {
        let slots: Vec<Arc<ModelSlot>> = self
            .slots
            .lock()
            .map_err(|_| LlmError::Internal {
                message: "model slot registry mutex poisoned".to_string(),
            })?
            .values()
            .cloned()
            .collect();

        Ok(slots.into_iter().filter(prewarm_vacant_slot).count())
    }

    /// Executes inference on an active slot.
    ///
    /// This auto-activates the slot if needed (Vacant -> Hibernated -> Active).
    ///
    /// # Errors
    ///
    /// Returns an error when the slot cannot be found, activated, or executed.
    pub async fn execute(&self, id: &ModelSlotId, input: ModelInput) -> LlmResult<ModelOutput> {
        let slot = self.get(id).ok_or_else(|| LlmError::Internal {
            message: format!("model slot not found: {id}"),
        })?;

        match slot.state() {
            SlotState::Active => slot.execute(input).await,
            SlotState::Hibernated => {
                // Auto-activate on demand
                self.activate(id)?;
                slot.execute(input).await
            }
            SlotState::Vacant => {
                // Hibernate then activate
                self.hibernate(id)?;
                self.activate(id)?;
                slot.execute(input).await
            }
        }
    }

    /// Prewarms a specific slot in the background.
    ///
    /// This is a "zero-copy landing" optimization that prepares the model
    /// for fast activation without blocking the current thread.
    ///
    /// Returns immediately, prewarming happens asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error when the slot cannot be found.
    pub fn prewarm_slot_background(&self, id: &ModelSlotId) -> LlmResult<()> {
        let slot = self.get(id).ok_or_else(|| LlmError::Internal {
            message: format!("model slot not found: {id}"),
        })?;

        // Transition to Hibernated if Vacant
        if slot.state() == SlotState::Vacant {
            let _ = slot.hibernate();
        }

        // Spawn background task for prewarming
        // Note: In a real implementation, this would spawn a task on a runtime
        // For now, we just mark the state transition
        tracing::debug!(
            event = "llm.runtime.bus.prewarm_background",
            slot_id = %id,
            "Background prewarm requested for slot"
        );

        Ok(())
    }
}

fn prewarm_vacant_slot(slot: &Arc<ModelSlot>) -> bool {
    if slot.state() != SlotState::Vacant {
        return false;
    }
    let _ = slot.hibernate();
    true
}

impl Default for ModelBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory trait for creating model executors.
pub trait ExecutorFactory: Send + Sync {
    /// Returns the executor ID this factory creates.
    fn id(&self) -> &ExecutorId;

    /// Creates an executor for the given metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when the executor cannot be constructed from the
    /// provided model metadata.
    fn create(&self, metadata: &ModelMetadata) -> LlmResult<Arc<Box<dyn ModelExecutor>>>;
}

// Global singleton
static MODEL_BUS: OnceLock<ModelBus> = OnceLock::new();

/// Returns the global model bus.
pub fn model_bus() -> &'static ModelBus {
    MODEL_BUS.get_or_init(ModelBus::new)
}

#[cfg(test)]
#[path = "../../tests/unit/runtime/bus.rs"]
mod tests;
