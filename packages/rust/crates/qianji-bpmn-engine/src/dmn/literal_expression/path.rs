use super::model::{NumericOperator, NumericPathExpression};
use crate::dmn_parse_api::parse_literal;
use crate::error::{BpmnEngineError, Result};
use serde_json::Value;

pub(super) fn parse_numeric_path_expression<'a>(
    source_id: &str,
    expression: &'a str,
) -> Result<Option<NumericPathExpression<'a>>> {
    let mut parts = expression.split_whitespace();
    let Some(path) = parts.next() else {
        return Ok(None);
    };
    let Some(operator) = parts.next() else {
        return Ok(None);
    };
    let Some(rhs) = parts.next() else {
        return Ok(None);
    };
    if parts.next().is_some() {
        return Ok(None);
    }
    let operator = match operator {
        "+" => NumericOperator::Add,
        "-" => NumericOperator::Subtract,
        _ => return Ok(None),
    };
    if !is_identifier_path(path) {
        return Err(unsupported_literal(source_id, expression));
    }
    let Value::Number(number) = parse_literal(source_id, rhs)? else {
        return Err(unsupported_literal(source_id, expression));
    };
    let Some(rhs) = number.as_f64() else {
        return Err(unsupported_literal(source_id, expression));
    };
    Ok(Some(NumericPathExpression {
        path,
        operator,
        rhs,
    }))
}

pub(super) fn resolve_json_path<'a>(variables: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = variables;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

pub(super) fn is_identifier_path(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(is_identifier_segment)
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub(super) fn unsupported_literal(source_id: &str, literal: &str) -> BpmnEngineError {
    BpmnEngineError::UnsupportedDmnLiteral {
        source_id: (source_id.to_string()).into(),
        literal: literal.to_string(),
    }
}
