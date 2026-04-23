//! A simple mock mechanism for testing and simulation.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;

/// A simple mock mechanism for testing and simulation.
pub struct MockMechanism {
    /// Friendly name of the mock node.
    pub name: String,
    /// Scheduling weight.
    pub weight: f32,
}

#[async_trait]
impl QianjiMechanism for MockMechanism {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(QianjiOutput {
            data: json!({ self.name.clone(): "done" }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        self.weight
    }
}
