use super::operand::{
    ComparisonOperator, ParsedBooleanPathCondition, parse_boolean_path_condition,
    parse_comparison_operator, resolve_boolean_variable_path,
};
use serde_json::Value;

/// Engine-owned counters exposed to bounded multi-instance completion
/// conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MultiInstanceCompletionCounts {
    /// Total planned iterations.
    pub(crate) total: u32,
    /// Completed iterations after the latest merge.
    pub(crate) completed: u32,
    /// Still-active iterations after the latest merge.
    pub(crate) active: u32,
}

/// Evaluation error for bounded multi-instance completion conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MultiInstanceCompletionConditionError {
    /// One boolean variable-path condition referenced a missing or non-boolean
    /// value at runtime.
    UnresolvedVariablePath(String),
    /// The source condition is outside the supported bounded subset.
    UnsupportedExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CounterName {
    Total,
    Completed,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComparisonTarget {
    Counter(CounterName),
    Literal(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedMultiInstanceCompletionCondition<'a> {
    BooleanPath {
        negated: bool,
        path: &'a str,
    },
    CounterComparison {
        lhs: CounterName,
        operator: ComparisonOperator,
        rhs: ComparisonTarget,
    },
}

/// Returns whether the source condition fits the bounded multi-instance
/// completion-condition subset.
pub(crate) fn is_supported_multi_instance_completion_condition(condition: &str) -> bool {
    parse_multi_instance_completion_condition(condition).is_some()
}

/// Evaluates one bounded multi-instance completion condition.
pub(crate) fn evaluate_multi_instance_completion_condition(
    condition: &str,
    variables: &Value,
    counts: MultiInstanceCompletionCounts,
) -> Result<bool, MultiInstanceCompletionConditionError> {
    let Some(parsed) = parse_multi_instance_completion_condition(condition) else {
        return Err(MultiInstanceCompletionConditionError::UnsupportedExpression);
    };

    match parsed {
        ParsedMultiInstanceCompletionCondition::BooleanPath { negated, path } => {
            let value = resolve_boolean_variable_path(variables, path).ok_or_else(|| {
                MultiInstanceCompletionConditionError::UnresolvedVariablePath(path.to_string())
            })?;
            Ok(if negated { !value } else { value })
        }
        ParsedMultiInstanceCompletionCondition::CounterComparison { lhs, operator, rhs } => {
            let lhs = resolve_counter(lhs, counts);
            let rhs = match rhs {
                ComparisonTarget::Counter(counter) => resolve_counter(counter, counts),
                ComparisonTarget::Literal(value) => value,
            };
            Ok(compare_counts(lhs, operator, rhs))
        }
    }
}

fn parse_multi_instance_completion_condition(
    condition: &str,
) -> Option<ParsedMultiInstanceCompletionCondition<'_>> {
    let trimmed = condition.trim();
    if trimmed.is_empty() {
        return None;
    }

    parse_counter_comparison(trimmed)
        .or_else(|| parse_boolean_path_condition(trimmed).map(parsed_multi_instance_boolean_path))
}

fn parse_counter_comparison(source: &str) -> Option<ParsedMultiInstanceCompletionCondition<'_>> {
    let mut parts = source.split_whitespace();
    let lhs = parse_counter_name(parts.next()?)?;
    let operator = parse_comparison_operator(parts.next()?)?;
    let rhs = parse_comparison_target(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }

    Some(ParsedMultiInstanceCompletionCondition::CounterComparison { lhs, operator, rhs })
}

fn parse_counter_name(source: &str) -> Option<CounterName> {
    match source {
        "total" | "nrOfInstances" => Some(CounterName::Total),
        "completed" | "nrOfCompletedInstances" => Some(CounterName::Completed),
        "active" | "nrOfActiveInstances" => Some(CounterName::Active),
        _ => None,
    }
}

fn parse_comparison_target(source: &str) -> Option<ComparisonTarget> {
    parse_counter_name(source)
        .map(ComparisonTarget::Counter)
        .or_else(|| source.parse::<u32>().ok().map(ComparisonTarget::Literal))
}

fn compare_counts(lhs: u32, operator: ComparisonOperator, rhs: u32) -> bool {
    match operator {
        ComparisonOperator::Eq => lhs == rhs,
        ComparisonOperator::Ne => lhs != rhs,
        ComparisonOperator::Lt => lhs < rhs,
        ComparisonOperator::Le => lhs <= rhs,
        ComparisonOperator::Gt => lhs > rhs,
        ComparisonOperator::Ge => lhs >= rhs,
    }
}

fn resolve_counter(counter: CounterName, counts: MultiInstanceCompletionCounts) -> u32 {
    match counter {
        CounterName::Total => counts.total,
        CounterName::Completed => counts.completed,
        CounterName::Active => counts.active,
    }
}

fn parsed_multi_instance_boolean_path(
    parsed: ParsedBooleanPathCondition<'_>,
) -> ParsedMultiInstanceCompletionCondition<'_> {
    ParsedMultiInstanceCompletionCondition::BooleanPath {
        negated: parsed.negated,
        path: parsed.path,
    }
}
