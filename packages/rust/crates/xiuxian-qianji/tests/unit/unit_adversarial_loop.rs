use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::executors::SynapseCalibrator;
use xiuxian_qianji::{
    FlowInstruction, QianjiEngine, QianjiMechanism, QianjiOutput, QianjiScheduler,
};

// Custom Mock that resolves itself after one retry
struct SelfResolvingProspector {
    pub retry_count: Arc<std::sync::atomic::AtomicU32>,
}

#[async_trait]
impl QianjiMechanism for SelfResolvingProspector {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let count = self
            .retry_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let drift = if count == 0 { 0.9 } else { 0.01 };

        Ok(QianjiOutput {
            data: json!({ "drift_score": drift, "prospector_done": true }),
            instruction: FlowInstruction::Continue,
        })
    }
    fn weight(&self) -> f32 {
        1.0
    }
}

#[tokio::test]
async fn test_adversarial_retry_loop_convergence() -> Result<(), xiuxian_qianji::error::QianjiError>
{
    let mut engine = QianjiEngine::new();

    let retry_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let prospector_mech = Arc::new(SelfResolvingProspector {
        retry_count: retry_count.clone(),
    });

    let calibrator_mech = Arc::new(SynapseCalibrator {
        target_node_id: "Prospector".to_string(),
        drift_threshold: 0.5,
    });

    let p = engine.add_mechanism("Prospector", prospector_mech);
    let c = engine.add_mechanism("Calibrator", calibrator_mech);

    engine.add_link(p, c, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);

    // Initial run triggers fail (0.9) -> retry -> success (0.01)
    let result = scheduler.run(json!({})).await?;

    assert_eq!(result["calibration_status"], "passed");
    assert_eq!(retry_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    Ok(())
}
