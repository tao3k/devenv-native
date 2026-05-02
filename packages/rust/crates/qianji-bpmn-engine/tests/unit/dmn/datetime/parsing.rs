use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnComparisonOperator, DmnDateTimeComparison, DmnDateTimeRange, DmnDateTimeRangeBound,
    DmnInputEntry, parse_dmn_decision,
};

#[test]
fn dmn_parser_supports_datetime_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("datetime-comparison-release-window.dmn"))
        .must("datetime comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DateTimeEquals("2026-04-20T09:00:00".into())
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

#[test]
fn dmn_parser_supports_offset_datetime_comparison_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-comparison-release-window-offset.dmn",
    ))
    .must("offset datetime comparison DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DateTimeEquals("2026-04-20T09:00:00Z".into())
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DateTimeComparison(DmnDateTimeComparison::new(
            DmnComparisonOperator::LessThan,
            "2026-04-21T00:00:00+00:00",
        ))
    );
    assert_eq!(
        decision.table.rules[2].input_entries[0],
        DmnInputEntry::DateTimeComparison(DmnDateTimeComparison::new(
            DmnComparisonOperator::GreaterThanOrEqual,
            "2026-04-21T00:00:00+00:00",
        ))
    );
}

#[test]
fn dmn_parser_supports_offset_datetime_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-range-maintenance-window-offset.dmn",
    ))
    .must("offset datetime range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DateTimeRange(DmnDateTimeRange::new(
            Some(DmnDateTimeRangeBound::new(
                "2026-05-01T09:00:00+09:00",
                true
            )),
            Some(DmnDateTimeRangeBound::new(
                "2026-05-01T12:00:00+09:00",
                false
            )),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DateTimeRange(DmnDateTimeRange::new(
            Some(DmnDateTimeRangeBound::new(
                "2026-05-01T13:00:00+09:00",
                true
            )),
            Some(DmnDateTimeRangeBound::new(
                "2026-05-01T15:00:00+09:00",
                true
            )),
        ))
    );
}
