use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;

/// Mechanism that retries a target node when observed drift exceeds a threshold.
pub struct SynapseCalibrator {
    /// Node id to retry when calibration fails.
    pub target_node_id: String,
    /// Drift threshold above which the mechanism requests a retry.
    pub drift_threshold: f32,
}

#[async_trait]
impl QianjiMechanism for SynapseCalibrator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // Logic: Extract evidence and claims from context
        // This is a simplified version of the Drift Calculation
        let drift_score = context
            .get("drift_score")
            .and_then(|value| serde_json::from_value::<f32>(value.clone()).ok())
            .unwrap_or(0.0);

        if drift_score > self.drift_threshold {
            Ok(QianjiOutput {
                data: json!({ "calibration_status": "failed", "reason": "Drift exceeds threshold" }),
                instruction: FlowInstruction::RetryNodes(vec![self.target_node_id.clone()]),
            })
        } else {
            Ok(QianjiOutput {
                data: json!({ "calibration_status": "passed" }),
                instruction: FlowInstruction::Continue,
            })
        }
    }

    fn weight(&self) -> f32 {
        10.0 // Calibration nodes usually have high priority
    }
}
