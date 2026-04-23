use super::{assert_dmn_json_snapshot, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, DmnDecisionRef, DmnEvaluationRequest, DmnHitPolicy, evaluate_dmn_decision,
    parse_dmn_decision, parse_dmn_decisions,
};
use serde_json::json;

#[test]
fn dmn_parser_single_decision_table_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source("simple-unique-eligibility.dmn"))
        .must("bounded DMN source should parse");

    assert_eq!(decision.table.hit_policy, DmnHitPolicy::Unique);
    assert_eq!(decision.table.inputs[0].lookup_path(), Some("tier"));
    assert_eq!(decision.table.inputs[0].type_ref.as_deref(), Some("string"));
    assert_eq!(
        decision.table.outputs[0].type_ref.as_deref(),
        Some("string")
    );
    assert_dmn_json_snapshot("simple_unique_eligibility_contract", &decision);
}

#[test]
fn dmn_parser_multiple_decisions_materialize_plural_contract() {
    let decisions = parse_dmn_decisions(&fixture_source("multiple-decisions.dmn"))
        .must("multi-decision DMN source should parse through the plural API");

    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].decision.decision_id.as_ref(), "loan-decision");
    assert_eq!(
        decisions[1].decision.decision_id.as_ref(),
        "secondary-review"
    );
    assert_eq!(decisions[0].source_id.as_ref(), "multiple-decisions.dmn");
}

#[test]
fn dmn_parser_exact_one_wrapper_rejects_multiple_decisions() {
    let error = parse_dmn_decision(&fixture_source("multiple-decisions.dmn"))
        .must_err("exact-one wrapper should stay explicit for multi-decision sources");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnDecisionCount {
            source_id: "multiple-decisions.dmn".to_string(),
            count: 2,
        }
    );
}

#[test]
fn dmn_parser_rejects_unsupported_unary_tests() {
    let error = parse_dmn_decision(&fixture_source("invalid-unsupported-unary-test.dmn"))
        .must_err("unsupported unary tests should stay explicit");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: "invalid-unsupported-unary-test.dmn".to_string(),
            expression: "duration(\"P1.5Y\")".to_string(),
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
