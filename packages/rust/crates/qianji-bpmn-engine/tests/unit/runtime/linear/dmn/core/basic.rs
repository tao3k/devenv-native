use super::super::assert_local_business_rule_task;
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
async fn runtime_business_rule_task_evaluates_registered_invocation_decision_locally() {
    assert_local_business_rule_task(
        "dmn_invocation_runtime",
        "wf_dmn_invocation_runtime",
        "Decision_invocation_runtime",
        "versioned-local-bkm-invocation-runtime-20191111.dmn",
        json!({ "applicant": { "age": 41 } }),
        json!({
            "applicant": { "age": 41 },
            "Decision_invocation_runtime": 42.0,
        }),
        75,
    )
    .await;
}
