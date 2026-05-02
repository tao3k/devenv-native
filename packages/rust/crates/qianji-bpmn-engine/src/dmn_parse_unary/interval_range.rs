//! DMN unary-test interval range parsing.

use super::bounds::{
    ensure_matching_duration_bound_families, parse_date_time_unary_value, parse_date_unary_value,
    parse_duration_unary_value, parse_numeric_value, parse_time_unary_value,
};
use crate::error::Result;
use crate::{
    DmnDateRange, DmnDateRangeBound, DmnDateTimeRange, DmnDateTimeRangeBound, DmnDurationRange,
    DmnDurationRangeBound, DmnInputEntry, DmnNumericRange, DmnNumericRangeBound, DmnTimeRange,
    DmnTimeRangeBound,
};

pub(super) fn parse_date_interval_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };
    if !(lower_raw.contains("date(") || upper_raw.contains("date(")) {
        return Ok(None);
    }

    let lower = parse_date_unary_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_date_unary_value(source_id, upper_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateRange(DmnDateRange::new(
        Some(DmnDateRangeBound::new(lower, first_char == '[')),
        Some(DmnDateRangeBound::new(upper, last_char == ']')),
    ))))
}

pub(super) fn parse_time_interval_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };
    if !(lower_raw.contains("time(") || upper_raw.contains("time(")) {
        return Ok(None);
    }

    let lower = parse_time_unary_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_time_unary_value(source_id, upper_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::TimeRange(DmnTimeRange::new(
        Some(DmnTimeRangeBound::new(lower, first_char == '[')),
        Some(DmnTimeRangeBound::new(upper, last_char == ']')),
    ))))
}

pub(super) fn parse_date_time_interval_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };
    if !(lower_raw.contains("date and time(") || upper_raw.contains("date and time(")) {
        return Ok(None);
    }

    let lower = parse_date_time_unary_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_date_time_unary_value(source_id, upper_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateTimeRange(DmnDateTimeRange::new(
        Some(DmnDateTimeRangeBound::new(lower, first_char == '[')),
        Some(DmnDateTimeRangeBound::new(upper, last_char == ']')),
    ))))
}

pub(super) fn parse_duration_interval_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };
    if !(lower_raw.contains("duration(") || upper_raw.contains("duration(")) {
        return Ok(None);
    }

    let lower = parse_duration_unary_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_duration_unary_value(source_id, upper_raw.trim(), raw)?;
    ensure_matching_duration_bound_families(source_id, raw, &lower, &upper)?;
    Ok(Some(DmnInputEntry::DurationRange(DmnDurationRange::new(
        Some(DmnDurationRangeBound::new(lower, first_char == '[')),
        Some(DmnDurationRangeBound::new(upper, last_char == ']')),
    ))))
}

pub(super) fn parse_numeric_interval_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };

    let lower = parse_numeric_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_numeric_value(source_id, upper_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::NumericRange(DmnNumericRange::new(
        Some(DmnNumericRangeBound::new(lower, first_char == '[')),
        Some(DmnNumericRangeBound::new(upper, last_char == ']')),
    ))))
}
