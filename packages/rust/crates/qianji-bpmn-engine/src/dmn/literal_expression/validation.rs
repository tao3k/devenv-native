use super::path::{is_identifier_path, parse_numeric_path_expression, unsupported_literal};
use crate::dmn_model_api::{DmnContextExpression, DmnRelationExpression};
use crate::dmn_parse_api::parse_literal;
use crate::error::{BpmnEngineError, Result};

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
