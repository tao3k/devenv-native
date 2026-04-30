use super::model::{NumericOperator, NumericPathExpression};
use super::path::{
    is_identifier_path, parse_numeric_path_expression, resolve_json_path, unsupported_literal,
};
use crate::dmn_parse_api::parse_literal;
use crate::error::Result;
use serde_json::{Number, Value};

pub(crate) fn evaluate_dmn_literal_expression(
    source_id: &str,
    expression: &str,
    variables: &Value,
) -> Result<Value> {
    let trimmed = expression.trim();
    if let Some(parsed) = parse_numeric_path_expression(source_id, trimmed)? {
        return evaluate_numeric_path_expression(source_id, trimmed, variables, &parsed);
    }
    if is_identifier_path(trimmed) {
        return resolve_json_path(variables, trimmed)
            .cloned()
            .ok_or_else(|| unsupported_literal(source_id, trimmed));
    }
    parse_literal(source_id, trimmed)
}

fn evaluate_numeric_path_expression(
    source_id: &str,
    expression: &str,
    variables: &Value,
    parsed: &NumericPathExpression<'_>,
) -> Result<Value> {
    let Some(left) = resolve_json_path(variables, parsed.path).and_then(Value::as_f64) else {
        return Err(unsupported_literal(source_id, expression));
    };
    let value = match parsed.operator {
        NumericOperator::Add => left + parsed.rhs,
        NumericOperator::Subtract => left - parsed.rhs,
    };
    let Some(number) = Number::from_f64(value) else {
        return Err(unsupported_literal(source_id, expression));
    };
    Ok(Value::Number(number))
}
