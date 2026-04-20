use super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnComparisonOperator, DmnDateTimeComparison, DmnDateTimeRange, DmnDateTimeRangeBound,
    DmnDecisionRef, DmnEvaluationRequest, DmnInputEntry, evaluate_dmn_decision, parse_dmn_decision,
};
use serde_json::json;

#[test]
fn dmn_parser_supports_datetime_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("datetime-comparison-release-window.dmn"))
        .must("datetime comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::Equals(json!("2026-04-20T09:00:00"))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DateTimeComparison(DmnDateTimeComparison::new(
            DmnComparisonOperator::LessThan,
            "2026-04-21T00:00:00",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DateTimeComparison(DmnDateTimeComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "2026-04-21T00:00:00",
        ))
    );
}

#[test]
fn dmn_parser_supports_datetime_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("datetime-range-maintenance-window.dmn"))
        .must("datetime range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DateTimeRange(DmnDateTimeRange::new(
            Some(DmnDateTimeRangeBound::new("2026-05-01T09:00:00", true)),
            Some(DmnDateTimeRangeBound::new("2026-05-01T12:00:00", false)),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DateTimeRange(DmnDateTimeRange::new(
            Some(DmnDateTimeRangeBound::new("2026-05-01T13:00:00", true)),
            Some(DmnDateTimeRangeBound::new("2026-05-01T15:00:00", true)),
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_datetime_comparisons() {
    let decision = parse_dmn_decision(&fixture_source("datetime-comparison-release-window.dmn"))
        .must("datetime comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-20T09:00:00" }),
        ),
    )
    .await
    .must("datetime comparison DMN evaluator should run");
    assert_eq!(exact.output, json!({ "phase": "launch-minute" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_launch_minute");

    let before = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-20T12:00:00" }),
        ),
    )
    .await
    .must("datetime comparison DMN evaluator should run");
    assert_eq!(before.output, json!({ "phase": "day-one" }));
    assert_eq!(before.matched_rule_ids[0].as_ref(), "rule_day_one");

    let after = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-21T00:00:00" }),
        ),
    )
    .await
    .must("datetime comparison DMN evaluator should run");
    assert_eq!(after.output, json!({ "phase": "post-day-one" }));
    assert_eq!(after.matched_rule_ids[0].as_ref(), "rule_post_day_one");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_datetime_ranges() {
    let decision = parse_dmn_decision(&fixture_source("datetime-range-maintenance-window.dmn"))
        .must("datetime range DMN source should parse");

    let morning = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T09:30:00" }),
        ),
    )
    .await
    .must("datetime range DMN evaluator should run");
    assert_eq!(morning.output, json!({ "window": "morning-maintenance" }));
    assert_eq!(morning.matched_rule_ids[0].as_ref(), "rule_morning_window");

    let afternoon = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T15:00:00" }),
        ),
    )
    .await
    .must("datetime range DMN evaluator should run");
    assert_eq!(
        afternoon.output,
        json!({ "window": "afternoon-maintenance" })
    );
    assert_eq!(
        afternoon.matched_rule_ids[0].as_ref(),
        "rule_afternoon_window"
    );

    let outside = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T12:30:00" }),
        ),
    )
    .await
    .must("datetime range DMN evaluator should run");
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}
