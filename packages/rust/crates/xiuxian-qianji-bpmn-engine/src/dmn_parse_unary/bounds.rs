//! DMN unary-test bound parsing helpers.

use super::literal::{
    parse_date_literal, parse_date_time_literal, parse_duration_literal, parse_literal,
    parse_time_literal,
};
use crate::dmn_duration::{
    DmnDurationFamily, parse_day_time_duration_str, parse_year_month_duration_str,
};
use crate::dmn_model_api::{DmnComparisonOperator, DmnNumericRangeBound};
use crate::error::{BpmnEngineError, Result};
use crate::{DmnDateRangeBound, DmnDateTimeRangeBound, DmnDurationRangeBound, DmnTimeRangeBound};

pub(super) fn parse_comparison_prefix(raw: &str) -> Option<(DmnComparisonOperator, &str)> {
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

pub(super) fn parse_left_question_mark_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnNumericRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_left_question_mark_date_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_left_question_mark_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnTimeRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_left_question_mark_date_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateTimeRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_left_question_mark_duration_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDurationRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnDurationRangeBound::new(
            parse_duration_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnDurationRangeBound::new(
            parse_duration_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_right_question_mark_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnNumericRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_right_question_mark_date_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_right_question_mark_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnTimeRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnTimeRangeBound::new(
            parse_time_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_right_question_mark_date_time_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateTimeRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnDateTimeRangeBound::new(
            parse_date_time_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_right_question_mark_duration_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDurationRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnDurationRangeBound::new(
            parse_duration_unary_value(source_id, value_raw.trim(), expression)?,
            true.into(),
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnDurationRangeBound::new(
            parse_duration_unary_value(source_id, value_raw.trim(), expression)?,
            false.into(),
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn parse_numeric_value(source_id: &str, raw: &str, expression: &str) -> Result<f64> {
    match parse_literal(source_id, raw)? {
        serde_json::Value::Number(number) => {
            number
                .as_f64()
                .ok_or_else(|| BpmnEngineError::UnsupportedDmnUnaryTest {
                    source_id: (source_id.to_string()).into(),
                    expression: expression.to_string(),
                })
        }
        _ => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        }),
    }
}

pub(super) fn parse_date_unary_value(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<String> {
    match parse_date_literal(source_id, raw)? {
        Some(date) => Ok(date),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        }),
    }
}

pub(super) fn parse_time_unary_value(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<String> {
    match parse_time_literal(source_id, raw)? {
        Some(time) => Ok(time),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        }),
    }
}

pub(super) fn parse_date_time_unary_value(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<String> {
    match parse_date_time_literal(source_id, raw)? {
        Some(date_time) => Ok(date_time),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        }),
    }
}

pub(super) fn parse_duration_unary_value(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<String> {
    match parse_duration_literal(source_id, raw)? {
        Some(duration) => Ok(duration),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        }),
    }
}

pub(super) fn ensure_matching_duration_bound_families(
    source_id: &str,
    expression: &str,
    left: &str,
    right: &str,
) -> Result<()> {
    let Some(left_family) = supported_duration_family(left) else {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        });
    };
    let Some(right_family) = supported_duration_family(right) else {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: (source_id.to_string()).into(),
            expression: expression.to_string(),
        });
    };
    if left_family == right_family {
        return Ok(());
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: (source_id.to_string()).into(),
        expression: expression.to_string(),
    })
}

pub(super) fn supported_duration_family(value: &str) -> Option<DmnDurationFamily> {
    if let Some(duration) = parse_day_time_duration_str(value) {
        return Some(duration.family());
    }
    parse_year_month_duration_str(value).map(crate::dmn_duration::DmnDurationValue::family)
}
