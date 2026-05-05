//! DMN unary-test parser dispatch implementation.

use super::comparison::{
    parse_date_comparison, parse_date_time_comparison, parse_duration_comparison,
    parse_numeric_comparison, parse_time_comparison,
};
use super::interval_range::{
    parse_date_interval_range, parse_date_time_interval_range, parse_duration_interval_range,
    parse_numeric_interval_range, parse_time_interval_range,
};
use super::literal::{parse_date_time_literal, parse_duration_literal, parse_literal};
use super::question_range::{
    parse_date_question_mark_range, parse_date_time_question_mark_range,
    parse_duration_question_mark_range, parse_numeric_question_mark_range,
    parse_time_question_mark_range,
};
use crate::{BpmnEngineError, DmnInputEntry};
use std::sync::Arc;

type Result<T> = std::result::Result<T, BpmnEngineError>;

pub(crate) fn parse_input_entry(source_id: &str, raw: &str) -> Result<DmnInputEntry> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return Ok(DmnInputEntry::Any);
    }
    if let Ok(Some(duration)) = parse_duration_literal(source_id, trimmed) {
        return Ok(DmnInputEntry::DurationEquals(Arc::<str>::from(duration)));
    }
    if let Ok(Some(date_time)) = parse_date_time_literal(source_id, trimmed) {
        return Ok(DmnInputEntry::DateTimeEquals(Arc::<str>::from(date_time)));
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
    if let Some(parsed) = parse_duration_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_duration_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_duration_interval_range(source_id, trimmed)? {
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
