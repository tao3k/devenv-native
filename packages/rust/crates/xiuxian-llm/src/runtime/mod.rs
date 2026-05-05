//! [AIP] Omni `ModelBus` Runtime Module.
//!
//! This module provides a unified model bus for hot-swap model management,
//! supporting Vision, Text, and Embedding modalities with three-state lifecycle:
//!
//! - **Vacant**: Only metadata, no memory footprint (< 1ms)
//! - **Hibernated**: mmap established, virtual memory mapped (< 100ms)
//! - **Active**: Inference ready, weights in physical memory
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      ModelBus (Global)                        │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                      Slots Registry                       ││
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       ││
//! │  │  │ Vision  │ │  Text   │ │ Embed   │ │   ...   │       ││
//! │  │  │  Slot   │ │  Slot   │ │  Slot   │ │         │       ││
//! │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                   Executor Factories                      ││
//! │  │  ┌────────────┐ ┌────────────┐ ┌────────────┐          ││
//! │  │  │  Generic   │ │    ...     │ │    ...     │          ││
//! │  │  └────────────┘ └────────────┘ └────────────┘          ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use xiuxian_llm::model_runtime::{model_bus, ModelSlotId, ModelInput};
//!
//! // Register a slot
//! let slot = model_bus().get(&ModelSlotId::new("generic-model"));
//!
//! // Execute inference (auto-activates if needed)
//! let output = model_bus().execute(&slot_id, ModelInput::Vision {
//!     prompt: "OCR this image".to_string(),
//!     images: vec![image_bytes],
//! }).await?;
//! ```

mod bus;
mod executor;
mod executors;
mod slot;

pub use bus::{ExecutorFactory, ModelBus, model_bus};
pub use executor::{ExecutorId, ModelExecutor, ModelInput, ModelOutput, NoopExecutor};
pub use slot::{ModelMetadata, ModelSlot, ModelSlotId, ResidencyToken, SlotState};
