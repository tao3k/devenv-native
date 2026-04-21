//! Suspend execution mechanism for human-in-the-loop workflows.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;

/// Mechanism responsible for suspending the workflow and returning control.
pub struct SuspendMechanism {
    /// The suspension reason code (e.g., "`waiting_for_approval`").
    pub reason: String,
    /// Message to prompt the user.
    pub prompt: String,
    /// If this key exists in the context, the suspension is bypassed.
    pub resume_key: Option<String>,
}

#[async_trait]
impl QianjiMechanism for SuspendMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // If the context already contains the resume_key, it means we have been resumed with the required data.
        if let Some(key) = &self.resume_key
            && context.get(key).is_some()
        {
            return Ok(QianjiOutput {
                data: json!({}),
                instruction: FlowInstruction::Continue,
            });
        }

        // Suspend aborts the local execution loop, but states are persisted via checkpointer.
        // The calling application interprets this specific reason as a pause, not a fatal failure.
        Ok(QianjiOutput {
            data: json!({ "suspend_prompt": self.prompt }),
            instruction: FlowInstruction::Suspend(self.reason.clone()),
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
