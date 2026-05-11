//! DMN unary-test literal parsing.

use crate::dmn_duration::{
    DmnDurationFamily, parse_day_time_duration_str, parse_year_month_duration_str,
};
use crate::error::{BpmnEngineError, Result};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};

pub(crate) fn parse_literal(source_id: &str, raw: &str) -> Result<serde_json::Value> {
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
    if let Some(duration) = parse_duration_literal(source_id, trimmed)? {
        return Ok(serde_json::Value::String(duration));
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
        source_id: (source_id.to_string()).into(),
        literal: trimmed.to_string(),
    })
}

pub(super) fn parse_date_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("date(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("date(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_exact_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    let date = NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| {
        BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        }
    })?;
    Ok(Some(date.to_string()))
}

pub(super) fn parse_time_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("time(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("time(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_exact_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    let time = NaiveTime::parse_from_str(&value, "%H:%M:%S").map_err(|_| {
        BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        }
    })?;
    Ok(Some(time.to_string()))
}

pub(super) fn parse_date_time_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("date and time(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("date and time(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_exact_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    validate_supported_date_time_literal(source_id, trimmed, &value)?;
    Ok(Some(value))
}

pub(super) fn parse_duration_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("duration(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("duration(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_exact_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: (source_id.to_string()).into(),
            literal: trimmed.to_string(),
        });
    };
    validate_supported_duration_literal(source_id, trimmed, &value)?;
    Ok(Some(value))
}

fn validate_supported_date_time_literal(source_id: &str, raw: &str, value: &str) -> Result<()> {
    if NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").is_ok() {
        return Ok(());
    }
    if DateTime::<FixedOffset>::parse_from_rfc3339(value).is_ok() {
        return Ok(());
    }
    Err(BpmnEngineError::UnsupportedDmnLiteral {
        source_id: (source_id.to_string()).into(),
        literal: raw.to_string(),
    })
}

fn validate_supported_duration_literal(
    source_id: &str,
    raw: &str,
    value: &str,
) -> Result<DmnDurationFamily> {
    if let Some(duration) = parse_day_time_duration_str(value) {
        return Ok(duration.family());
    }
    if let Some(duration) = parse_year_month_duration_str(value) {
        return Ok(duration.family());
    }
    Err(BpmnEngineError::UnsupportedDmnLiteral {
        source_id: (source_id.to_string()).into(),
        literal: raw.to_string(),
    })
}

pub(super) fn parse_quoted_string(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return Some(raw[1..raw.len() - 1].to_string());
    }
    None
}

pub(super) fn parse_exact_quoted_string(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let quote = bytes[0];
    if !matches!(quote, b'"' | b'\'') || bytes[bytes.len() - 1] != quote {
        return None;
    }
    let inner = &raw[1..raw.len() - 1];
    if inner.as_bytes().contains(&quote) {
        return None;
    }
    Some(inner.to_string())
}

fn is_bare_string_token(raw: &str) -> bool {
    raw.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))
}
