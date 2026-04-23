//! Bounded direct DMN literal-expression runtime.

use crate::dmn_model_api::{
    DmnContextExpression, DmnDecisionDefinition, DmnEvaluationResult, DmnListExpression,
    DmnLiteralExpression, DmnRelationExpression,
};
use crate::dmn_parse_api::parse_literal;
use crate::error::{BpmnEngineError, Result};
use serde_json::{Map, Number, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericOperator {
    Add,
    Subtract,
}

struct NumericPathExpression<'a> {
    path: &'a str,
    operator: NumericOperator,
    rhs: f64,
}

pub(crate) fn evaluate_dmn_literal_expression_decision(
    decision: &DmnDecisionDefinition,
    literal: &DmnLiteralExpression,
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    let value = evaluate_dmn_literal_expression(
        decision.source_id.as_ref(),
        literal.text.as_ref(),
        variables,
    )?;
    let mut output = Map::new();
    output.insert(decision.decision.decision_id.to_string(), value);
    Ok(DmnEvaluationResult::new(
        decision.decision.decision_id.as_ref(),
        Value::Object(output),
        Vec::new(),
    ))
}

pub(crate) fn evaluate_dmn_list_expression_decision(
    decision: &DmnDecisionDefinition,
    list: &DmnListExpression,
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    let mut values = Vec::with_capacity(list.items.len());
    for item in &list.items {
        values.push(evaluate_dmn_literal_expression(
            decision.source_id.as_ref(),
            item.text.as_ref(),
            variables,
        )?);
    }
    let mut output = Map::new();
    output.insert(
        decision.decision.decision_id.to_string(),
        Value::Array(values),
    );
    Ok(DmnEvaluationResult::new(
        decision.decision.decision_id.as_ref(),
        Value::Object(output),
        Vec::new(),
    ))
}

pub(crate) fn evaluate_dmn_context_expression_decision(
    decision: &DmnDecisionDefinition,
    context: &DmnContextExpression,
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    validate_dmn_context_expression_syntax(decision.source_id.as_ref(), context)?;

    let mut scope = variables.as_object().cloned().unwrap_or_default();
    let mut context_values = Map::new();
    let mut final_value = None;
    for (index, entry) in context.entries.iter().enumerate() {
        let value = evaluate_dmn_literal_expression(
            decision.source_id.as_ref(),
            entry.expression.text.as_ref(),
            &Value::Object(scope.clone()),
        )?;
        match entry.variable_name.as_deref() {
            Some(variable_name) => {
                scope.insert(variable_name.to_string(), value.clone());
                context_values.insert(variable_name.to_string(), value);
            }
            None if index + 1 == context.entries.len() => {
                final_value = Some(value);
            }
            None => {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "evaluate_dmn_context_non_final_result_entry",
                });
            }
        }
    }

    let mut output = Map::new();
    output.insert(
        decision.decision.decision_id.to_string(),
        final_value.unwrap_or(Value::Object(context_values)),
    );
    Ok(DmnEvaluationResult::new(
        decision.decision.decision_id.as_ref(),
        Value::Object(output),
        Vec::new(),
    ))
}

pub(crate) fn evaluate_dmn_relation_expression_decision(
    decision: &DmnDecisionDefinition,
    relation: &DmnRelationExpression,
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    validate_dmn_relation_expression_syntax(decision.source_id.as_ref(), relation)?;

    let mut rows = Vec::with_capacity(relation.rows.len());
    for row in &relation.rows {
        let mut row_output = Map::new();
        for (column, cell) in relation.columns.iter().zip(&row.cells) {
            let value = evaluate_dmn_literal_expression(
                decision.source_id.as_ref(),
                cell.text.as_ref(),
                variables,
            )?;
            row_output.insert(column.output_key().to_string(), value);
        }
        rows.push(Value::Object(row_output));
    }

    let mut output = Map::new();
    output.insert(
        decision.decision.decision_id.to_string(),
        Value::Array(rows),
    );
    Ok(DmnEvaluationResult::new(
        decision.decision.decision_id.as_ref(),
        Value::Object(output),
        Vec::new(),
    ))
}

pub(crate) fn validate_dmn_literal_expression_syntax(
    source_id: &str,
    expression: &str,
) -> Result<()> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return Err(unsupported_literal(source_id, expression));
    }
    if parse_numeric_path_expression(source_id, trimmed)?.is_some() || is_identifier_path(trimmed) {
        return Ok(());
    }
    parse_literal(source_id, trimmed).map(drop)
}

pub(crate) fn validate_dmn_context_expression_syntax(
    source_id: &str,
    context: &DmnContextExpression,
) -> Result<()> {
    if context.entries.is_empty() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "validate_dmn_context_empty",
        });
    }

    for (index, entry) in context.entries.iter().enumerate() {
        validate_dmn_literal_expression_syntax(source_id, entry.expression.text.as_ref())?;
        match entry.variable_name.as_deref() {
            Some(variable_name) if is_identifier_path(variable_name) => {}
            Some(_) => {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "validate_dmn_context_invalid_variable_name",
                });
            }
            None if index + 1 == context.entries.len() => {}
            None => {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "validate_dmn_context_non_final_result_entry",
                });
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_dmn_relation_expression_syntax(
    source_id: &str,
    relation: &DmnRelationExpression,
) -> Result<()> {
    if relation.columns.is_empty() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "validate_dmn_relation_empty_columns",
        });
    }
    for row in &relation.rows {
        if row.cells.len() != relation.columns.len() {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "validate_dmn_relation_row_arity",
            });
        }
        for cell in &row.cells {
            validate_dmn_literal_expression_syntax(source_id, cell.text.as_ref())?;
        }
    }
    Ok(())
}

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

fn parse_numeric_path_expression<'a>(
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

fn resolve_json_path<'a>(variables: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = variables;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn is_identifier_path(path: &str) -> bool {
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

fn unsupported_literal(source_id: &str, literal: &str) -> BpmnEngineError {
    BpmnEngineError::UnsupportedDmnLiteral {
        source_id: source_id.to_string(),
        literal: literal.to_string(),
    }
}
