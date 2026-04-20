//! Shared bounded repeat-condition parsing and evaluation helpers.

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
enum ComparisonOperator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
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

    if let Some(parsed) = parse_counter_comparison(trimmed) {
        return Some(parsed);
    }

    let (negated, path) = match trimmed.strip_prefix("not ") {
        Some(path) => (true, path.trim()),
        None => (false, trimmed),
    };
    if is_identifier_path(path) {
        return Some(ParsedMultiInstanceCompletionCondition::BooleanPath { negated, path });
    }

    None
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

fn parse_comparison_operator(source: &str) -> Option<ComparisonOperator> {
    match source {
        "==" => Some(ComparisonOperator::Eq),
        "!=" => Some(ComparisonOperator::Ne),
        "<" => Some(ComparisonOperator::Lt),
        "<=" => Some(ComparisonOperator::Le),
        ">" => Some(ComparisonOperator::Gt),
        ">=" => Some(ComparisonOperator::Ge),
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

fn resolve_boolean_variable_path(variables: &Value, path: &str) -> Option<bool> {
    let mut current = variables;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    current.as_bool()
}

fn is_identifier_path(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(is_identifier_segment)
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
