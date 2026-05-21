use super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnComparisonOperator, DmnDecisionRef, DmnEvaluationRequest, DmnInputEntry, DmnTimeComparison,
    DmnTimeRange, DmnTimeRangeBound, evaluate_dmn_decision, parse_dmn_decision,
};
use serde_json::json;

#[test]
fn dmn_parser_supports_time_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("time-comparison-business-hours.dmn"))
        .must("time comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::Equals(json!("09:00:00"))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::TimeComparison(DmnTimeComparison::new(
            DmnComparisonOperator::LessThan,
            "09:00:00",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::TimeComparison(DmnTimeComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "17:00:00",
        ))
    );
}

#[test]
fn dmn_parser_supports_time_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("time-range-shift-window.dmn"))
        .must("time range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::TimeRange(DmnTimeRange::new(
            Some(DmnTimeRangeBound::new("09:00:00", true.into())),
            Some(DmnTimeRangeBound::new("12:00:00", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::TimeRange(DmnTimeRange::new(
            Some(DmnTimeRangeBound::new("13:00:00", true.into())),
            Some(DmnTimeRangeBound::new("15:00:00", true.into())),
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_time_comparisons() {
    let decision = parse_dmn_decision(&fixture_source("time-comparison-business-hours.dmn"))
        .must("time comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("business-hours")
                .with_source_id("time-comparison-business-hours.dmn"),
            json!({ "check_time": "09:00:00" }),
        ),
    )
    .await
    .must("time comparison DMN evaluator should run");
    assert_eq!(exact.output, json!({ "state": "opening-bell" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_opening_bell");

    let before = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("business-hours")
                .with_source_id("time-comparison-business-hours.dmn"),
            json!({ "check_time": "08:30:00" }),
        ),
    )
    .await
    .must("time comparison DMN evaluator should run");
    assert_eq!(before.output, json!({ "state": "before-open" }));
    assert_eq!(before.matched_rule_ids[0].as_ref(), "rule_before_open");

    let after = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("business-hours")
                .with_source_id("time-comparison-business-hours.dmn"),
            json!({ "check_time": "17:00:00" }),
        ),
    )
    .await
    .must("time comparison DMN evaluator should run");
    assert_eq!(after.output, json!({ "state": "after-close" }));
    assert_eq!(after.matched_rule_ids[0].as_ref(), "rule_after_close");

    let during = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("business-hours")
                .with_source_id("time-comparison-business-hours.dmn"),
            json!({ "check_time": "12:00:00" }),
        ),
    )
    .await
    .must("time comparison DMN evaluator should run");
    assert_eq!(during.output, json!({ "state": "business-hours" }));
    assert_eq!(during.matched_rule_ids[0].as_ref(), "rule_business_hours");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_time_ranges() {
    let decision = parse_dmn_decision(&fixture_source("time-range-shift-window.dmn"))
        .must("time range DMN source should parse");

    let morning = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("shift-window").with_source_id("time-range-shift-window.dmn"),
            json!({ "shift_time": "09:30:00" }),
        ),
    )
    .await
    .must("time range DMN evaluator should run");
    assert_eq!(morning.output, json!({ "window": "morning-window" }));
    assert_eq!(morning.matched_rule_ids[0].as_ref(), "rule_morning_window");

    let afternoon = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("shift-window").with_source_id("time-range-shift-window.dmn"),
            json!({ "shift_time": "15:00:00" }),
        ),
    )
    .await
    .must("time range DMN evaluator should run");
    assert_eq!(afternoon.output, json!({ "window": "afternoon-window" }));
    assert_eq!(
        afternoon.matched_rule_ids[0].as_ref(),
        "rule_afternoon_window"
    );

    let outside = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("shift-window").with_source_id("time-range-shift-window.dmn"),
            json!({ "shift_time": "12:30:00" }),
        ),
    )
    .await
    .must("time range DMN evaluator should run");
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}
