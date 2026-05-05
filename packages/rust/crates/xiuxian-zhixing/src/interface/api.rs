//! Secure action-selection API exposed to agent/application layers.

use crate::Result;
use serde::{Deserialize, Serialize};

/// Defined secure actions that an LLM can trigger.
///
/// This follows the "Action-Selector Pattern" to prevent arbitrary command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecureAction {
    /// Change the status of a specific task.
    UpdateTaskStatus {
        /// Unique ID of the task.
        id: uuid::Uuid,
        /// New status to apply.
        status: crate::agenda::Status,
    },
    /// Change the scheduled date or deadline of a task.
    RescheduleTask {
        /// Unique ID of the task.
        id: uuid::Uuid,
        /// New date/time for the task.
        new_date: chrono::DateTime<chrono::Utc>,
    },
    /// Add a new insight or note to a task, linking it to the knowledge graph.
    CreateInsight {
        /// Unique ID of the task.
        task_id: uuid::Uuid,
        /// Content of the insight.
        content: String,
    },
    /// Inform the user that the task input is ambiguous and needs clarification.
    RequestRefinement {
        /// Unique ID of the task.
        task_id: uuid::Uuid,
        /// Reason for the refinement request.
        reason: String,
    },
}

/// Interface for the LLM to interact with Zhixing logic.
///
/// The actual implementation (calling LLM APIs) will reside in the agent/application layer
/// to keep this crate decoupled from specific LLM providers.
pub trait ZhixingLlmInterface: Send + Sync {
    /// Given a natural language context, the LLM selects a predefined [`SecureAction`].
    ///
    /// # Errors
    /// Returns an error if the LLM cannot be reached or fails to parse.
    fn select_action(&self, prompt_context: &str) -> Result<SecureAction>;

    /// Given raw journal text, the LLM compiles it into structured metadata and insights.
    ///
    /// # Errors
    /// Returns an error if the LLM cannot be reached or fails to parse.
    fn compile_reflection(&self, raw_text: &str) -> Result<String>;
}
