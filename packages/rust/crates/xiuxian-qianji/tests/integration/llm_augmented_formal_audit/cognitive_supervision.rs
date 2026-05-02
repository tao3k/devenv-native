use super::{
    SequencedMockLlmClient, make_test_mechanism, must_bool, must_f64, must_object, must_ok,
};
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism};

#[tokio::test]
async fn llm_augmented_audit_includes_cognitive_metrics_when_enabled() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.95</score><reason>excellent</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, true);

    let output = must_ok(
        mechanism
            .execute(&json!({
                "raw_facts": "Test agenda with balanced workload.",
                "request": "Critique this agenda."
            }))
            .await,
        "cognitive supervision execution should succeed",
    );

    assert!(output.data.get("_cognitive_coherence").is_some());
    assert!(output.data.get("_early_halt_triggered").is_some());
    assert!(output.data.get("_cognitive_distribution").is_some());

    let coherence = must_f64(
        &output.data["_cognitive_coherence"],
        "coherence should be numeric",
    );
    assert!(
        (0.0..=1.0).contains(&coherence),
        "coherence should be in [0, 1], got {coherence}"
    );

    let distribution = must_object(
        &output.data["_cognitive_distribution"],
        "cognitive distribution should be an object",
    );
    assert!(distribution.contains_key("meta"));
    assert!(distribution.contains_key("operational"));
    assert!(distribution.contains_key("epistemic"));
    assert!(distribution.contains_key("instrumental"));
    assert!(distribution.contains_key("balance"));
    assert!(distribution.contains_key("uncertainty_ratio"));
}

#[tokio::test]
async fn llm_augmented_audit_without_cognitive_supervision() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.95</score><reason>good</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, false);

    let output = must_ok(
        mechanism
            .execute(&json!({
                "raw_facts": "Test agenda.",
                "request": "Critique."
            }))
            .await,
        "non-cognitive execution should succeed",
    );

    assert!(output.data.get("_cognitive_coherence").is_none());
    assert!(output.data.get("_early_halt_triggered").is_none());
    assert!(output.data.get("_cognitive_distribution").is_none());
    assert_eq!(output.data["audit_status"], "passed");
}

#[tokio::test]
async fn llm_augmented_audit_early_halt_triggers_abort() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.95</score><reason>passed</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, true);

    let output = must_ok(
        mechanism
            .execute(&json!({
                "raw_facts": "Test agenda.",
                "request": "Critique."
            }))
            .await,
        "early halt verification should succeed",
    );

    let early_halt = must_bool(
        &output.data["_early_halt_triggered"],
        "early_halt_triggered should be boolean",
    );

    if early_halt {
        assert!(
            matches!(output.instruction, FlowInstruction::Abort(_)),
            "early_halt_triggered=true should result in Abort instruction"
        );
    }
}

#[tokio::test]
async fn llm_augmented_audit_cognitive_distribution_values_in_range() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.90</score><reason>good</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, true);

    let output = must_ok(
        mechanism
            .execute(&json!({
                "raw_facts": "Test agenda.",
                "request": "Critique."
            }))
            .await,
        "distribution range execution should succeed",
    );

    let distribution = must_object(
        &output.data["_cognitive_distribution"],
        "cognitive distribution should be an object",
    );

    for (key, value) in distribution {
        if let Some(score) = value.as_f64() {
            assert!(
                (0.0..=1.0).contains(&score),
                "dimension {key} should be in [0, 1], got {score}"
            );
        }
    }
}
