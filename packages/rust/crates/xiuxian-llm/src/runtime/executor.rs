//! [AIP] Unified Model Executor Trait for Omni `ModelBus`.
//!
//! This module defines the core abstraction for model execution,
//! supporting Text, Vision, and Embedding modalities with hot-swap capability.

use std::sync::Arc;

use async_trait::async_trait;

use crate::llm::error::LlmResult;

/// Input types for model execution.
#[derive(Debug, Clone)]
pub enum ModelInput {
    /// Text input for language models.
    Text(String),
    /// Vision input with prompt and images.
    Vision {
        /// Text prompt for the vision task.
        prompt: String,
        /// Prepared images for inference.
        images: Vec<Vec<u8>>,
    },
    /// Text batch for embedding generation.
    Embedding(Vec<String>),
}

/// Output types from model execution.
#[derive(Debug, Clone)]
pub enum ModelOutput {
    /// Text output from language models.
    Text(String),
    /// Vision output (markdown format).
    Vision(String),
    /// Embedding vectors.
    Embedding(Vec<Vec<f32>>),
}

/// Unique identifier for a model executor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecutorId(Arc<str>);

impl ExecutorId {
    /// Creates a new executor ID.
    #[must_use]
    pub fn new(id: impl Into<Self>) -> Self {
        id.into()
    }

    /// Returns the ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ExecutorId {
    fn from(id: &str) -> Self {
        Self(Arc::from(id.to_string()))
    }
}

impl From<String> for ExecutorId {
    fn from(id: String) -> Self {
        Self(Arc::from(id))
    }
}

impl std::fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Core trait for model executors in the Omni `ModelBus`.
///
/// Implementations must support hot-swap via `prewarm()` and state transitions.
#[async_trait]
pub trait ModelExecutor: Send + Sync {
    /// Returns the unique identifier for this executor.
    fn id(&self) -> &ExecutorId;

    /// Returns the human-readable name for this executor.
    fn name(&self) -> &'static str;

    /// Executes the model with the given input.
    ///
    /// # Errors
    ///
    /// Returns an error when the executor cannot produce an output for the
    /// requested input.
    async fn execute(&self, input: ModelInput) -> LlmResult<ModelOutput>;

    /// Prewarms the executor for faster first inference.
    ///
    /// This is called by the `ModelBus` when transitioning from Hibernated to Active.
    ///
    /// # Errors
    ///
    /// Returns an error when the executor cannot prepare its runtime state.
    fn prewarm(&self) -> LlmResult<()>;

    /// Returns the memory footprint in bytes.
    fn memory_bytes(&self) -> u64;

    /// Returns whether this executor is currently loaded and ready.
    fn is_ready(&self) -> bool;
}

/// A no-op executor that is always ready but uses no memory.
/// Used for testing and as a placeholder.
pub struct NoopExecutor {
    id: ExecutorId,
    name: &'static str,
    ready: bool,
}

impl NoopExecutor {
    /// Creates a new no-op executor.
    #[must_use]
    pub fn new(id: impl Into<ExecutorId>) -> Self {
        Self {
            id: ExecutorId::new(id),
            name: "noop",
            ready: true,
        }
    }
}

#[async_trait]
impl ModelExecutor for NoopExecutor {
    fn id(&self) -> &ExecutorId {
        &self.id
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn execute(&self, _input: ModelInput) -> LlmResult<ModelOutput> {
        Ok(ModelOutput::Text("noop".to_string()))
    }

    fn prewarm(&self) -> LlmResult<()> {
        Ok(())
    }

    fn memory_bytes(&self) -> u64 {
        0
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}
