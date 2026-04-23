use super::{assert_local_business_rule_task, run_local_business_rule_task};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::BpmnEngineError;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_dmn_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local",
        "wf_dmn_local",
        "loan-decision",
        "simple-unique-eligibility.dmn",
        json!({ "tier": "gold" }),
        json!({ "tier": "gold", "approval": "approve" }),
        70,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_literal_expression_decision_locally() {
    assert_local_business_rule_task(
        "dmn_literal",
        "wf_dmn_literal",
        "Decision_literal_expression",
        "versioned-literal-expression-decision-20191111.dmn",
        json!({ "applicant": { "age": 41 } }),
        json!({
            "applicant": { "age": 41 },
            "Decision_literal_expression": 42.0,
        }),
        71,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_list_expression_decision_locally() {
    assert_local_business_rule_task(
        "dmn_list",
        "wf_dmn_list",
        "Decision_list",
        "versioned-list-decision-20191111.dmn",
        json!({ "request_id": "r1" }),
        json!({
            "request_id": "r1",
            "Decision_list": ["approve", "review"],
        }),
        72,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_context_expression_decision_locally() {
    assert_local_business_rule_task(
        "dmn_context",
        "wf_dmn_context",
        "Decision_context",
        "versioned-context-decision-20191111.dmn",
        json!({ "applicant": { "age": 41 } }),
        json!({
            "applicant": { "age": 41 },
            "Decision_context": 42.0,
        }),
        73,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_relation_expression_decision_locally() {
    assert_local_business_rule_task(
        "dmn_relation",
        "wf_dmn_relation",
        "Decision_relation",
        "versioned-relation-decision-20191111.dmn",
        json!({ "request_id": "r1" }),
        json!({
            "request_id": "r1",
            "Decision_relation": [
                { "lender": "Lender A", "rate": 3.95 },
                { "lender": "Lender B", "rate": 4.10 },
            ],
        }),
        74,
    )
    .await;
}

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
        75,
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
        76,
    )
    .await
    .must_err("missing required decision targets should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnRequiredDecisionTarget {
            source_id: "versioned-missing-required-decision-runtime-20191111.dmn".to_string(),
            decision_id: "Decision_missing_dependency".to_string(),
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
        77,
    )
    .await
    .must_err("cyclic required decision dependencies should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::CyclicDmnRequiredDecisionDependency {
            source_id: "versioned-cyclic-required-decision-runtime-20191111.dmn".to_string(),
            decision_id: "Decision_a".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_binds_required_input_alias_locally() {
    assert_local_business_rule_task(
        "dmn_required_input",
        "wf_dmn_required_input",
        "Decision_alias_required_input",
        "versioned-local-required-input-runtime-20191111.dmn",
        json!({ "applicant_input": { "age": 41 } }),
        json!({
            "applicant_input": { "age": 41 },
            "approval": "approve",
        }),
        78,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_missing_required_input_target_locally() {
    let error = run_local_business_rule_task(
        "dmn_required_input_missing",
        "wf_dmn_required_input_missing",
        "Decision_missing_required_input",
        "versioned-missing-required-input-runtime-20191111.dmn",
        json!({ "applicant_input": { "age": 41 } }),
        79,
    )
    .await
    .must_err("missing required input targets should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnRequiredInputTarget {
            source_id: "versioned-missing-required-input-runtime-20191111.dmn".to_string(),
            decision_id: "Decision_missing_required_input".to_string(),
            href: "#InputData_missing".to_string(),
        }
    );
}
