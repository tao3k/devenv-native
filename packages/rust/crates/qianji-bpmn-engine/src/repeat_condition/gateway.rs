use super::common::{
    ComparisonOperator, ParsedBooleanPathCondition, compare_numbers, is_identifier_path,
    parse_boolean_path_condition, parse_comparison_operator, parse_numeric_literal,
    resolve_boolean_variable_path, resolve_numeric_variable_path,
};
use serde_json::Value;

/// Structured parse summary for one bounded gateway condition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GatewayConditionSummary {
    /// One boolean variable path, optionally negated by `not`.
    BooleanPath {
        /// Whether the condition uses `not`.
        negated: bool,
        /// Variable path resolved at runtime.
        path: String,
    },
    /// One numeric variable-path comparison against a finite numeric literal.
    NumericComparison {
        /// Left-hand variable path resolved at runtime.
        lhs: String,
        /// Comparison operator as written in the bounded expression.
        operator: String,
        /// Right-hand numeric literal.
        rhs: f64,
    },
}

/// Evaluation error for bounded exclusive-gateway conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GatewayConditionError {
    /// One bounded gateway condition referenced a missing or incompatible
    /// variable value at runtime.
    UnresolvedVariablePath(String),
    /// The source condition is outside the supported bounded subset.
    UnsupportedExpression,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParsedGatewayCondition<'a> {
    BooleanPath {
        negated: bool,
        path: &'a str,
    },
    NumericComparison {
        lhs: &'a str,
        operator: ComparisonOperator,
        rhs: f64,
    },
}

/// Returns whether the source condition fits the bounded exclusive-gateway
/// subset.
pub(crate) fn is_supported_gateway_condition(condition: &str) -> bool {
    parse_gateway_condition(condition).is_some()
}

/// Parses one bounded exclusive-gateway condition into a structured summary.
#[must_use]
pub fn parse_gateway_condition_summary(condition: &str) -> Option<GatewayConditionSummary> {
    match parse_gateway_condition(condition)? {
        ParsedGatewayCondition::BooleanPath { negated, path } => {
            Some(GatewayConditionSummary::BooleanPath {
                negated,
                path: path.to_string(),
            })
        }
        ParsedGatewayCondition::NumericComparison { lhs, operator, rhs } => {
            Some(GatewayConditionSummary::NumericComparison {
                lhs: lhs.to_string(),
                operator: comparison_operator_source(operator).to_string(),
                rhs,
            })
        }
    }
}

/// Evaluates one bounded exclusive-gateway condition.
pub(crate) fn evaluate_gateway_condition(
    condition: &str,
    variables: &Value,
) -> Result<bool, GatewayConditionError> {
    let Some(parsed) = parse_gateway_condition(condition) else {
        return Err(GatewayConditionError::UnsupportedExpression);
    };

    match parsed {
        ParsedGatewayCondition::BooleanPath { negated, path } => {
            let value = resolve_boolean_variable_path(variables, path)
                .ok_or_else(|| GatewayConditionError::UnresolvedVariablePath(path.to_string()))?;
            Ok(if negated { !value } else { value })
        }
        ParsedGatewayCondition::NumericComparison { lhs, operator, rhs } => {
            let lhs = resolve_numeric_variable_path(variables, lhs)
                .ok_or_else(|| GatewayConditionError::UnresolvedVariablePath(lhs.to_string()))?;
            Ok(compare_numbers(lhs, operator, rhs))
        }
    }
}

fn parse_gateway_condition(condition: &str) -> Option<ParsedGatewayCondition<'_>> {
    let trimmed = condition.trim();
    if trimmed.is_empty() {
        return None;
    }

    parse_gateway_numeric_comparison(trimmed)
        .or_else(|| parse_boolean_path_condition(trimmed).map(parsed_gateway_boolean_path))
}

fn parse_gateway_numeric_comparison(source: &str) -> Option<ParsedGatewayCondition<'_>> {
    let mut parts = source.split_whitespace();
    let lhs = parts.next()?;
    let operator = parse_comparison_operator(parts.next()?)?;
    let rhs = parse_numeric_literal(parts.next()?)?;
    if parts.next().is_some() || !is_identifier_path(lhs) {
        return None;
    }

    Some(ParsedGatewayCondition::NumericComparison { lhs, operator, rhs })
}

fn parsed_gateway_boolean_path(
    parsed: ParsedBooleanPathCondition<'_>,
) -> ParsedGatewayCondition<'_> {
    ParsedGatewayCondition::BooleanPath {
        negated: parsed.negated,
        path: parsed.path,
    }
}

fn comparison_operator_source(operator: ComparisonOperator) -> &'static str {
    match operator {
        ComparisonOperator::Eq => "==",
        ComparisonOperator::Ne => "!=",
        ComparisonOperator::Lt => "<",
        ComparisonOperator::Le => "<=",
        ComparisonOperator::Gt => ">",
        ComparisonOperator::Ge => ">=",
    }
}
