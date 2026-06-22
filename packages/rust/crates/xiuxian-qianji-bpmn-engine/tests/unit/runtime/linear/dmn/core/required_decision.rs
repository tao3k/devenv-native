use crate::runtime::linear::dmn::{assert_local_business_rule_task, run_local_business_rule_task};
use crate::test_support::MustExt as _;
use serde_json::json;
use xiuxian_qianji_bpmn_engine::BpmnEngineError;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_resolves_required_decision_dependency_locally() {
    assert_local_business_rule_task(
        "dmn_required_decision",
        "wf_dmn_required_decision",
        "Decision_approval",
        "versioned-local-required-decision-runtime-20191111.dmn",
        json!({ "tier": "gold", "customerScore": 720 }),
        json!({
            "tier": "gold",
            "customerScore": 720,
            "approval": "approve",
        }),
        76,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_missing_required_decision_target_locally() {
    let error = run_local_business_rule_task(
        "dmn_required_decision_missing",
        "wf_dmn_required_decision_missing",
        "Decision_missing_dependency",
        "versioned-missing-required-decision-runtime-20191111.dmn",
        json!({ "seed": "x" }),
        77,
    )
    .await
    .must_err("missing required decision targets should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnRequiredDecisionTarget {
            source_id: ("versioned-missing-required-decision-runtime-20191111.dmn".to_string())
                .into(),
            decision_id: ("Decision_missing_dependency".to_string()).into(),
            href: "#Decision_missing".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_cyclic_required_decision_dependency_locally() {
    let error = run_local_business_rule_task(
        "dmn_required_decision_cycle",
        "wf_dmn_required_decision_cycle",
        "Decision_a",
        "versioned-cyclic-required-decision-runtime-20191111.dmn",
        json!({ "seed": "x" }),
        78,
    )
    .await
    .must_err("cyclic required decision dependencies should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::CyclicDmnRequiredDecisionDependency {
            source_id: ("versioned-cyclic-required-decision-runtime-20191111.dmn".to_string())
                .into(),
            decision_id: ("Decision_a".to_string()).into(),
        }
    );
}
