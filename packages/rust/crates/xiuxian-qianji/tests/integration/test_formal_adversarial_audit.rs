#![cfg(feature = "wendao-integration")]

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::{FlowInstruction, QianjiMechanism, QianjiOutput, QianjiScheduler};

// A slightly smarter Mock that "learns" from audit failures
struct SelfHealingMock;

#[async_trait]
impl QianjiMechanism for SelfHealingMock {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // If audit has failed once, provide a grounded trace
        let audit_status = context.get("audit_status").and_then(|v| v.as_str());

        let trace = if audit_status == Some("failed") {
            json!([{ "predicate": "Fixed", "has_grounding": true, "confidence": 1.0 }])
        } else {
            json!([{ "predicate": "Buggy", "has_grounding": false, "confidence": 0.5 }])
        };

        Ok(QianjiOutput {
            data: json!({ "analysis_trace": trace }),
            instruction: FlowInstruction::Continue,
        })
    }
    fn weight(&self) -> f32 {
        1.0
    }
}

#[tokio::test]
async fn test_formal_adversarial_audit_convergence() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = xiuxian_qianji::QianjiEngine::new();
    let analyzer = Arc::new(SelfHealingMock);
    let skeptic = Arc::new(xiuxian_qianji::executors::FormalAuditMechanism {
        invariants: vec![xiuxian_qianji::safety::logic::Invariant::MustBeGrounded],
        retry_target_ids: vec!["Analyzer".to_string()],
    });

    let a = engine.add_mechanism("Analyzer", analyzer);
    let s = engine.add_mechanism("Skeptic", skeptic);
    engine.add_link(a, s, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler.run(json!({})).await?;

    assert_eq!(result["audit_status"], "passed");
    let Some(trace) = result["analysis_trace"].as_array() else {
        return Err(std::io::Error::other("analysis_trace should be an array").into());
    };
    assert_eq!(trace[0]["predicate"], "Fixed");
    Ok(())
}
