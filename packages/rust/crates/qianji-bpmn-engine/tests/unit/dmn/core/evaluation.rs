use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, DmnDecisionRef, DmnEvaluationRequest, evaluate_dmn_decision,
    parse_dmn_decision,
};
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_direct_literal_expression_returns_decision_keyed_output() {
    let decision = parse_dmn_decision(&fixture_source(
        "versioned-literal-expression-decision-20191111.dmn",
    ))
    .must("bounded direct literal-expression DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("Decision_literal_expression")
            .with_source_id("versioned-literal-expression-decision-20191111.dmn"),
        json!({ "applicant": { "age": 41 } }),
    );

    let result = evaluate_dmn_decision(&decision, &request)
        .await
        .must("bounded direct literal-expression evaluator should run");

    assert_eq!(result.decision_id.as_ref(), "Decision_literal_expression");
    assert_eq!(
        result.output,
        json!({ "Decision_literal_expression": 42.0 })
    );
    assert!(result.matched_rule_ids.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_direct_list_expression_returns_decision_keyed_output() {
    let decision = parse_dmn_decision(&fixture_source("versioned-list-decision-20191111.dmn"))
        .must("bounded direct list DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("Decision_list").with_source_id("versioned-list-decision-20191111.dmn"),
        json!({}),
    );

    let result = evaluate_dmn_decision(&decision, &request)
        .await
        .must("bounded direct list evaluator should run");

    assert_eq!(result.decision_id.as_ref(), "Decision_list");
    assert_eq!(
        result.output,
        json!({ "Decision_list": ["approve", "review"] })
    );
    assert!(result.matched_rule_ids.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_direct_context_expression_returns_decision_keyed_output() {
    let decision = parse_dmn_decision(&fixture_source("versioned-context-decision-20191111.dmn"))
        .must("bounded direct context DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("Decision_context")
            .with_source_id("versioned-context-decision-20191111.dmn"),
        json!({ "applicant": { "age": 41 } }),
    );

    let result = evaluate_dmn_decision(&decision, &request)
        .await
        .must("bounded direct context evaluator should run");

    assert_eq!(result.decision_id.as_ref(), "Decision_context");
    assert_eq!(result.output, json!({ "Decision_context": 42.0 }));
    assert!(result.matched_rule_ids.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_direct_relation_expression_returns_decision_keyed_output() {
    let decision = parse_dmn_decision(&fixture_source("versioned-relation-decision-20191111.dmn"))
        .must("bounded direct relation DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("Decision_relation")
            .with_source_id("versioned-relation-decision-20191111.dmn"),
        json!({}),
    );

    let result = evaluate_dmn_decision(&decision, &request)
        .await
        .must("bounded direct relation evaluator should run");

    assert_eq!(result.decision_id.as_ref(), "Decision_relation");
    assert_eq!(
        result.output,
        json!({
            "Decision_relation": [
                { "lender": "Lender A", "rate": 3.95 },
                { "lender": "Lender B", "rate": 4.10 },
            ],
        })
    );
    assert!(result.matched_rule_ids.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_direct_invocation_requires_package_context() {
    let decision = parse_dmn_decision(&fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ))
    .must("bounded direct invocation DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("Decision_invocation")
            .with_source_id("versioned-invocation-decision-20191111.dmn"),
        json!({ "applicant": { "age": 41 } }),
    );

    let error = evaluate_dmn_decision(&decision, &request)
        .await
        .must_err("direct invocation should require package-owned BKM context");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedOperation {
            operation: "evaluate_dmn_invocation_without_package_context",
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_returns_first_matching_rule_output() {
    let decision = parse_dmn_decision(&fixture_source("simple-unique-eligibility.dmn"))
        .must("bounded DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("loan-decision").with_source_id("simple-unique-eligibility.dmn"),
        json!({ "tier": "gold" }),
    );

    let result = evaluate_dmn_decision(&decision, &request)
        .await
        .must("bounded DMN evaluator should run");

    assert_eq!(result.decision_id.as_ref(), "loan-decision");
    assert_eq!(result.output, json!({ "approval": "approve" }));
    assert_eq!(result.matched_rule_ids.len(), 1);
    assert_eq!(result.matched_rule_ids[0].as_ref(), "rule_approve");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_collect_accumulates_matching_outputs() {
    let decision = parse_dmn_decision(&fixture_source("collect-risk-tags.dmn"))
        .must("bounded DMN collect source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("risk-tags").with_source_id("collect-risk-tags.dmn"),
        json!({ "amount": 20 }),
    );

    let result = evaluate_dmn_decision(&decision, &request)
        .await
        .must("bounded DMN collect evaluator should run");

    assert_eq!(result.output, json!({ "tags": ["medium", "needs-review"] }));
    assert_eq!(result.matched_rule_ids.len(), 2);
    assert_eq!(result.matched_rule_ids[0].as_ref(), "rule_medium");
    assert_eq!(result.matched_rule_ids[1].as_ref(), "rule_review");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_rejects_mismatched_decision_reference() {
    let decision = parse_dmn_decision(&fixture_source("simple-unique-eligibility.dmn"))
        .must("bounded DMN source should parse");
    let request = DmnEvaluationRequest::new(
        DmnDecisionRef::new("other-decision").with_source_id("simple-unique-eligibility.dmn"),
        json!({ "tier": "gold" }),
    );

    let error = evaluate_dmn_decision(&decision, &request)
        .await
        .must_err("wrong decision reference should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::DmnDecisionMismatch {
            expected: "loan-decision".to_string(),
            actual: "other-decision".to_string(),
        }
    );
}
