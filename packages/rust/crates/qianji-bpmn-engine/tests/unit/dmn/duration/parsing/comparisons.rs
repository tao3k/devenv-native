use super::super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnComparisonOperator, DmnDurationComparison, DmnInputEntry, parse_dmn_decision,
};

#[test]
fn dmn_parser_supports_duration_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("duration-comparison-sla-window.dmn"))
        .must("duration comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationEquals("PT30M".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::LessThan,
            "PT1H",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "PT1H",
        ))
    );
}

#[test]
fn dmn_parser_supports_year_month_duration_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "year-month-duration-comparison-retention-window.dmn",
    ))
    .must("year-month duration comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationEquals("P6M".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::LessThan,
            "P1Y",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "P1Y",
        ))
    );
}

#[test]
fn dmn_parser_supports_negative_duration_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "negative-duration-comparison-recovery-window.dmn",
    ))
    .must("negative duration comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationEquals("-PT30M".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::LessThan,
            "PT0S",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "PT0S",
        ))
    );
}

#[test]
fn dmn_parser_supports_fractional_second_duration_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "fractional-duration-comparison-subsecond-window.dmn",
    ))
    .must("fractional-second duration comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationEquals("PT1.5S".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::LessThan,
            "PT2.25S",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "PT2.25S",
        ))
    );
}

#[test]
fn dmn_parser_supports_fractional_day_time_unit_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "fractional-duration-comparison-hour-window.dmn",
    ))
    .must("fractional day-time unit comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationEquals("PT1.5H".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::LessThan,
            "PT2.25H",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "PT2.25H",
        ))
    );
}

#[test]
fn dmn_parser_supports_comma_day_time_unit_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("comma-duration-comparison-hour-window.dmn"))
        .must("comma day-time unit comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationEquals("PT1,5H".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::LessThan,
            "PT2,25H",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DurationComparison(DmnDurationComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "PT2,25H",
        ))
    );
}
