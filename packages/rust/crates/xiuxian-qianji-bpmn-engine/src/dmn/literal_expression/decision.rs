use super::evaluator::evaluate_dmn_literal_expression;
use super::validation::{
    validate_dmn_context_expression_syntax, validate_dmn_relation_expression_syntax,
};
use crate::dmn_model_api::{
    DmnContextExpression, DmnDecisionDefinition, DmnEvaluationResult, DmnListExpression,
    DmnLiteralExpression, DmnRelationExpression,
};
use crate::error::{BpmnEngineError, Result};
use serde_json::{Map, Value};

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
