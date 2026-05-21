use super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateRangeBound, DmnDecisionRef,
    DmnEvaluationRequest, DmnInputEntry, evaluate_dmn_decision, parse_dmn_decision,
};
use serde_json::json;

#[test]
fn dmn_parser_supports_date_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("date-comparison-effective-date.dmn"))
        .must("date comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::Equals(json!("2026-01-01"))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DateComparison(DmnDateComparison::new(
            DmnComparisonOperator::LessThan,
            "2026-02-01",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DateComparison(DmnDateComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "2026-02-01",
        ))
    );
}

#[test]
fn dmn_parser_supports_date_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("date-range-review-window.dmn"))
        .must("date range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DateRange(DmnDateRange::new(
            Some(DmnDateRangeBound::new("2026-03-01", true.into())),
            Some(DmnDateRangeBound::new("2026-03-10", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DateRange(DmnDateRange::new(
            Some(DmnDateRangeBound::new("2026-04-01", true.into())),
            Some(DmnDateRangeBound::new("2026-04-05", true.into())),
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_date_comparisons() {
    let decision = parse_dmn_decision(&fixture_source("date-comparison-effective-date.dmn"))
        .must("date comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("effective-date-band")
                .with_source_id("date-comparison-effective-date.dmn"),
            json!({ "effective_date": "2026-01-01" }),
        ),
    )
    .await
    .must("date comparison DMN evaluator should run");
    assert_eq!(exact.output, json!({ "phase": "launch-day" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_launch_day");

    let before = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("effective-date-band")
                .with_source_id("date-comparison-effective-date.dmn"),
            json!({ "effective_date": "2026-01-15" }),
        ),
    )
    .await
    .must("date comparison DMN evaluator should run");
    assert_eq!(before.output, json!({ "phase": "pre-cutoff" }));
    assert_eq!(before.matched_rule_ids[0].as_ref(), "rule_pre_cutoff");

    let after = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("effective-date-band")
                .with_source_id("date-comparison-effective-date.dmn"),
            json!({ "effective_date": "2026-02-01" }),
        ),
    )
    .await
    .must("date comparison DMN evaluator should run");
    assert_eq!(after.output, json!({ "phase": "cutoff-or-later" }));
    assert_eq!(after.matched_rule_ids[0].as_ref(), "rule_cutoff_or_later");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_date_ranges() {
    let decision = parse_dmn_decision(&fixture_source("date-range-review-window.dmn"))
        .must("date range DMN source should parse");

    let march = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("review-window").with_source_id("date-range-review-window.dmn"),
            json!({ "review_date": "2026-03-05" }),
        ),
    )
    .await
    .must("date range DMN evaluator should run");
    assert_eq!(march.output, json!({ "window": "march-window" }));
    assert_eq!(march.matched_rule_ids[0].as_ref(), "rule_march_window");

    let april = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("review-window").with_source_id("date-range-review-window.dmn"),
            json!({ "review_date": "2026-04-05" }),
        ),
    )
    .await
    .must("date range DMN evaluator should run");
    assert_eq!(april.output, json!({ "window": "april-window" }));
    assert_eq!(april.matched_rule_ids[0].as_ref(), "rule_april_window");

    let outside = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("review-window").with_source_id("date-range-review-window.dmn"),
            json!({ "review_date": "2026-05-01" }),
        ),
    )
    .await
    .must("date range DMN evaluator should run");
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}
