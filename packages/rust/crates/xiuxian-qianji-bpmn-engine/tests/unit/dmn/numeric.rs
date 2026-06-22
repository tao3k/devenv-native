use super::fixture_source;
use crate::test_support::MustExt as _;
use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    DmnComparisonOperator, DmnDecisionRef, DmnEvaluationRequest, DmnInputEntry,
    DmnNumericRangeBound, evaluate_dmn_decision, parse_dmn_decision,
};

#[test]
fn dmn_parser_supports_numeric_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("numeric-comparison-age-band.dmn"))
        .must("comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::Equals(json!(30))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::NumericComparison(xiuxian_qianji_bpmn_engine::DmnNumericComparison::new(
            DmnComparisonOperator::LessThan,
            25.0,
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::NumericComparison(xiuxian_qianji_bpmn_engine::DmnNumericComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            25.0,
        ))
    );
}

#[test]
fn dmn_parser_supports_numeric_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("numeric-range-age-window.dmn"))
        .must("range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::NumericRange(xiuxian_qianji_bpmn_engine::DmnNumericRange::new(
            Some(DmnNumericRangeBound::new(100.0, true.into())),
            Some(DmnNumericRangeBound::new(110.0, true.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::NumericRange(xiuxian_qianji_bpmn_engine::DmnNumericRange::new(
            Some(DmnNumericRangeBound::new(200.0, true.into())),
            Some(DmnNumericRangeBound::new(210.0, false.into())),
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_numeric_comparisons() {
    let decision = parse_dmn_decision(&fixture_source("numeric-comparison-age-band.dmn"))
        .must("comparison DMN source should parse");

    let low = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("age-band").with_source_id("numeric-comparison-age-band.dmn"),
            json!({ "age": 24 }),
        ),
    )
    .await
    .must("numeric comparison DMN evaluator should run");
    assert_eq!(low.output, json!({ "band": "low" }));
    assert_eq!(low.matched_rule_ids[0].as_ref(), "rule_low");

    let high = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("age-band").with_source_id("numeric-comparison-age-band.dmn"),
            json!({ "age": 25 }),
        ),
    )
    .await
    .must("numeric comparison DMN evaluator should run");
    assert_eq!(high.output, json!({ "band": "high" }));
    assert_eq!(high.matched_rule_ids[0].as_ref(), "rule_high");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("age-band").with_source_id("numeric-comparison-age-band.dmn"),
            json!({ "age": 30 }),
        ),
    )
    .await
    .must("numeric comparison DMN evaluator should run");
    assert_eq!(exact.output, json!({ "band": "exact-30" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_exact_30");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_numeric_ranges() {
    let decision = parse_dmn_decision(&fixture_source("numeric-range-age-window.dmn"))
        .must("range DMN source should parse");

    let inclusive = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("age-window").with_source_id("numeric-range-age-window.dmn"),
            json!({ "age": 100 }),
        ),
    )
    .await
    .must("numeric range DMN evaluator should run");
    assert_eq!(inclusive.output, json!({ "window": "inclusive-window" }));
    assert_eq!(
        inclusive.matched_rule_ids[0].as_ref(),
        "rule_inclusive_window"
    );

    let interval = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("age-window").with_source_id("numeric-range-age-window.dmn"),
            json!({ "age": 205 }),
        ),
    )
    .await
    .must("numeric range DMN evaluator should run");
    assert_eq!(interval.output, json!({ "window": "interval-window" }));
    assert_eq!(
        interval.matched_rule_ids[0].as_ref(),
        "rule_interval_window"
    );

    let outside = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("age-window").with_source_id("numeric-range-age-window.dmn"),
            json!({ "age": 210 }),
        ),
    )
    .await
    .must("numeric range DMN evaluator should run");
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}
