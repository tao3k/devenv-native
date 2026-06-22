//! DMN unary-test comparison parsing.

use super::bounds::{
    parse_comparison_prefix, parse_date_time_unary_value, parse_date_unary_value,
    parse_duration_unary_value, parse_numeric_value, parse_time_unary_value,
};
use crate::error::Result;
use crate::{
    DmnDateComparison, DmnDateTimeComparison, DmnDurationComparison, DmnInputEntry,
    DmnNumericComparison, DmnTimeComparison,
};

pub(super) fn parse_date_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    if !value_raw.trim().starts_with("date(") {
        return Ok(None);
    }
    let value = parse_date_unary_value(source_id, value_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateComparison(DmnDateComparison::new(
        operator, value,
    ))))
}

pub(super) fn parse_time_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    if !value_raw.trim().starts_with("time(") {
        return Ok(None);
    }
    let value = parse_time_unary_value(source_id, value_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::TimeComparison(DmnTimeComparison::new(
        operator, value,
    ))))
}

pub(super) fn parse_date_time_comparison(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    if !value_raw.trim().starts_with("date and time(") {
        return Ok(None);
    }
    let value = parse_date_time_unary_value(source_id, value_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateTimeComparison(
        DmnDateTimeComparison::new(operator, value),
    )))
}

pub(super) fn parse_duration_comparison(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    if !value_raw.trim().starts_with("duration(") {
        return Ok(None);
    }
    let value = parse_duration_unary_value(source_id, value_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DurationComparison(
        DmnDurationComparison::new(operator, value),
    )))
}

pub(super) fn parse_numeric_comparison(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    let value = parse_numeric_value(source_id, value_raw, raw)?;
    Ok(Some(DmnInputEntry::NumericComparison(
        DmnNumericComparison::new(operator, value),
    )))
}
