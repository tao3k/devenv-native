use crate::dmn_model_api::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateRangeBound,
    DmnDateTimeComparison, DmnDateTimeRange, DmnDateTimeRangeBound, DmnInputEntry,
    DmnNumericComparison, DmnNumericRange, DmnNumericRangeBound, DmnTimeComparison, DmnTimeRange,
    DmnTimeRangeBound,
};
use crate::error::{BpmnEngineError, Result};
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;

pub(super) fn parse_input_entry(source_id: &str, raw: &str) -> Result<DmnInputEntry> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return Ok(DmnInputEntry::Any);
    }
    if let Ok(literal) = parse_literal(source_id, trimmed) {
        return Ok(DmnInputEntry::Equals(literal));
    }
    if let Some(parsed) = parse_date_time_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_time_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_time_interval_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_interval_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_time_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_time_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_time_interval_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_numeric_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_numeric_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_numeric_interval_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: trimmed.to_string(),
    })
}

pub(super) fn parse_literal(source_id: &str, raw: &str) -> Result<serde_json::Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    if let Some(date) = parse_date_literal(source_id, trimmed)? {
        return Ok(serde_json::Value::String(date));
    }
    if let Some(date_time) = parse_date_time_literal(source_id, trimmed)? {
        return Ok(serde_json::Value::String(date_time));
    }
    if let Some(time) = parse_time_literal(source_id, trimmed)? {
        return Ok(serde_json::Value::String(time));
    }
    if let Some(value) = parse_quoted_string(trimmed) {
        return Ok(serde_json::Value::String(value));
    }
    match trimmed {
        "true" => return Ok(serde_json::Value::Bool(true)),
        "false" => return Ok(serde_json::Value::Bool(false)),
        "null" => return Ok(serde_json::Value::Null),
        _ => {}
    }
    if let Ok(number) = serde_json::from_str::<serde_json::Number>(trimmed) {
        return Ok(serde_json::Value::Number(number));
    }
    if is_bare_string_token(trimmed) {
        return Ok(serde_json::Value::String(trimmed.to_string()));
    }
    Err(BpmnEngineError::UnsupportedDmnLiteral {
        source_id: source_id.to_string(),
        literal: trimmed.to_string(),
    })
}

fn parse_date_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("date(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("date(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let date = NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| {
        BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        }
    })?;
    Ok(Some(date.to_string()))
}

fn parse_time_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("time(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("time(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let time = NaiveTime::parse_from_str(&value, "%H:%M:%S").map_err(|_| {
        BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        }
    })?;
    Ok(Some(time.to_string()))
}

fn parse_date_time_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("date and time(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("date and time(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let date_time = NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S").map_err(|_| {
        BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        }
    })?;
    let _ = date_time;
    Ok(Some(value))
}

fn parse_date_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_time_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_date_time_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_numeric_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    let value = parse_numeric_value(source_id, value_raw, raw)?;
    Ok(Some(DmnInputEntry::NumericComparison(
        DmnNumericComparison::new(operator, value),
    )))
}

fn parse_date_question_mark_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
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

fn parse_time_question_mark_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
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

fn parse_date_time_question_mark_range(
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
            source_id: source_id.to_string(),
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

fn parse_numeric_question_mark_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
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

fn parse_date_interval_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_time_interval_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_date_time_interval_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_numeric_interval_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
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

fn parse_comparison_prefix(raw: &str) -> Option<(DmnComparisonOperator, &str)> {
    if let Some(value) = raw.strip_prefix("<=") {
        return Some((DmnComparisonOperator::LessThanOrEqual, value.trim()));
    }
    if let Some(value) = raw.strip_prefix(">=") {
        return Some((DmnComparisonOperator::GreaterThanOrEqual, value.trim()));
    }
    if let Some(value) = raw.strip_prefix('<') {
        return Some((DmnComparisonOperator::LessThan, value.trim()));
    }
    raw.strip_prefix('>')
        .map(|value| (DmnComparisonOperator::GreaterThan, value.trim()))
}

fn parse_left_question_mark_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnNumericRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_left_question_mark_date_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_left_question_mark_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnTimeRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_left_question_mark_date_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateTimeRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_right_question_mark_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnNumericRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_right_question_mark_date_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_right_question_mark_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnTimeRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_right_question_mark_date_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateTimeRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_numeric_value(source_id: &str, raw: &str, expression: &str) -> Result<f64> {
    match parse_literal(source_id, raw)? {
        serde_json::Value::Number(number) => {
            number
                .as_f64()
                .ok_or_else(|| BpmnEngineError::UnsupportedDmnUnaryTest {
                    source_id: source_id.to_string(),
                    expression: expression.to_string(),
                })
        }
        _ => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: expression.to_string(),
        }),
    }
}

fn parse_date_unary_value(source_id: &str, raw: &str, expression: &str) -> Result<String> {
    match parse_date_literal(source_id, raw)? {
        Some(date) => Ok(date),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: expression.to_string(),
        }),
    }
}

fn parse_time_unary_value(source_id: &str, raw: &str, expression: &str) -> Result<String> {
    match parse_time_literal(source_id, raw)? {
        Some(time) => Ok(time),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: expression.to_string(),
        }),
    }
}

fn parse_date_time_unary_value(source_id: &str, raw: &str, expression: &str) -> Result<String> {
    match parse_date_time_literal(source_id, raw)? {
        Some(date_time) => Ok(date_time),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: expression.to_string(),
        }),
    }
}

fn parse_quoted_string(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return Some(raw[1..raw.len() - 1].to_string());
    }
    None
}

fn is_bare_string_token(raw: &str) -> bool {
    raw.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))
}
