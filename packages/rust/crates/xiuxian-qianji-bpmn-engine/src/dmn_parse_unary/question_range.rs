//! DMN unary-test question-mark range parsing.

use super::bounds::{
    ensure_matching_duration_bound_families, parse_left_question_mark_bound,
    parse_left_question_mark_date_bound, parse_left_question_mark_date_time_bound,
    parse_left_question_mark_duration_bound, parse_left_question_mark_time_bound,
    parse_right_question_mark_bound, parse_right_question_mark_date_bound,
    parse_right_question_mark_date_time_bound, parse_right_question_mark_duration_bound,
    parse_right_question_mark_time_bound,
};
use crate::error::{BpmnEngineError, Result};
use crate::{
    DmnDateRange, DmnDateTimeRange, DmnDurationRange, DmnInputEntry, DmnNumericRange, DmnTimeRange,
};

pub(super) fn parse_date_question_mark_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: raw.to_string(),
        });
    }
    if !(left_raw.contains("date(") || right_raw.contains("date(")) {
        return Ok(None);
    }

    let lower = parse_left_question_mark_date_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_date_bound(source_id, right_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateRange(DmnDateRange::new(
        Some(lower),
        Some(upper),
    ))))
}

pub(super) fn parse_time_question_mark_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: raw.to_string(),
        });
    }
    if !(left_raw.contains("time(") || right_raw.contains("time(")) {
        return Ok(None);
    }

    let lower = parse_left_question_mark_time_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_time_bound(source_id, right_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::TimeRange(DmnTimeRange::new(
        Some(lower),
        Some(upper),
    ))))
}

pub(super) fn parse_date_time_question_mark_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: raw.to_string(),
        });
    }
    if !(left_raw.contains("date and time(") || right_raw.contains("date and time(")) {
        return Ok(None);
    }

    let lower = parse_left_question_mark_date_time_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_date_time_bound(source_id, right_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateTimeRange(DmnDateTimeRange::new(
        Some(lower),
        Some(upper),
    ))))
}

pub(super) fn parse_duration_question_mark_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: raw.to_string(),
        });
    }
    if !(left_raw.contains("duration(") || right_raw.contains("duration(")) {
        return Ok(None);
    }

    let lower = parse_left_question_mark_duration_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_duration_bound(source_id, right_raw.trim(), raw)?;
    ensure_matching_duration_bound_families(source_id, raw, &lower.value, &upper.value)?;
    Ok(Some(DmnInputEntry::DurationRange(DmnDurationRange::new(
        Some(lower),
        Some(upper),
    ))))
}

pub(super) fn parse_numeric_question_mark_range(
    source_id: &str,
    raw: &str,
) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: raw.to_string(),
        });
    }

    let lower = parse_left_question_mark_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_bound(source_id, right_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::NumericRange(DmnNumericRange::new(
        Some(lower),
        Some(upper),
    ))))
}
