use crate::contracts::{FlowInstruction, QianjiMechanism};
use crate::executors::FormalAuditMechanism;
use crate::safety::logic::Invariant;
use serde_json::json;

#[tokio::test]
async fn test_formal_audit_passes() {
    let audit = FormalAuditMechanism {
        invariants: vec![Invariant::MustBeGrounded],
        retry_target_ids: vec!["Analyzer".to_string()],
    };

    let context = json!({
        "analysis_trace": [
            {
                "predicate": "A implies B",
                "has_grounding": true,
                "confidence": 0.95
            }
        ]
    });

    let output = audit
        .execute(&context)
        .await
        .unwrap_or_else(|err| panic!("formal audit should pass: {err}"));
    assert_eq!(output.data["audit_status"], "passed");
    match output.instruction {
        FlowInstruction::Continue => {}
        _ => panic!("Expected Continue instruction"),
    }
}

#[tokio::test]
async fn test_formal_audit_fails() {
    let audit = FormalAuditMechanism {
        invariants: vec![Invariant::MustBeGrounded],
        retry_target_ids: vec!["Analyzer".to_string()],
    };

    let context = json!({
        "analysis_trace": [
            {
                "predicate": "A implies B",
                "has_grounding": false,
                "confidence": 0.95
            }
        ]
    });

    let output = audit
        .execute(&context)
        .await
        .unwrap_or_else(|err| panic!("formal audit should produce retry output: {err}"));
    assert_eq!(output.data["audit_status"], "failed");
    match output.instruction {
        FlowInstruction::RetryNodes(nodes) => {
            assert_eq!(nodes[0], "Analyzer");
        }
        _ => panic!("Expected RetryNodes instruction"),
    }
}
