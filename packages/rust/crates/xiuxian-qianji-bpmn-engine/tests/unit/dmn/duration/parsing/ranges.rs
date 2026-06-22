use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    DmnDurationRange, DmnDurationRangeBound, DmnInputEntry, parse_dmn_decision,
};

#[test]
fn dmn_parser_supports_duration_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source("duration-range-review-delay.dmn"))
        .must("duration range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("PT15M", true.into())),
            Some(DmnDurationRangeBound::new("PT45M", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("P1DT1H", true.into())),
            Some(DmnDurationRangeBound::new("P1DT2H", true.into())),
        ))
    );
}

#[test]
fn dmn_parser_supports_year_month_duration_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "year-month-duration-range-contract-term.dmn",
    ))
    .must("year-month duration range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("P6M", true.into())),
            Some(DmnDurationRangeBound::new("P1Y", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("P1Y", true.into())),
            Some(DmnDurationRangeBound::new("P2Y", true.into())),
        ))
    );
}

#[test]
fn dmn_parser_supports_fractional_second_duration_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "fractional-duration-range-subsecond-window.dmn",
    ))
    .must("fractional-second duration range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("-PT0.5S", true.into())),
            Some(DmnDurationRangeBound::new("PT0.5S", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("PT1.25S", true.into())),
            Some(DmnDurationRangeBound::new("PT2.75S", true.into())),
        ))
    );
}

#[test]
fn dmn_parser_supports_fractional_day_time_unit_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "fractional-duration-range-day-minute-window.dmn",
    ))
    .must("fractional day-time unit range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("PT1.5M", true.into())),
            Some(DmnDurationRangeBound::new("PT2.75M", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("P1.25D", true.into())),
            Some(DmnDurationRangeBound::new("P1.5D", true.into())),
        ))
    );
}

#[test]
fn dmn_parser_supports_comma_day_time_unit_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "comma-duration-range-day-minute-window.dmn",
    ))
    .must("comma day-time unit range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("PT1,5M", true.into())),
            Some(DmnDurationRangeBound::new("PT2,75M", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("P1,25D", true.into())),
            Some(DmnDurationRangeBound::new("P1,5D", true.into())),
        ))
    );
}

#[test]
fn dmn_parser_supports_negative_year_month_duration_range_unary_tests() {
    let decision = parse_dmn_decision(&fixture_source(
        "negative-year-month-duration-range-account-window.dmn",
    ))
    .must("negative year-month duration range DMN source should parse");

    assert_eq!(
        decision.table.rules[0].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("-P1Y", true.into())),
            Some(DmnDurationRangeBound::new("-P6M", false.into())),
        ))
    );
    assert_eq!(
        decision.table.rules[1].input_entries[0],
        DmnInputEntry::DurationRange(DmnDurationRange::new(
            Some(DmnDurationRangeBound::new("-P6M", true.into())),
            Some(DmnDurationRangeBound::new("P0M", true.into())),
        ))
    );
}
