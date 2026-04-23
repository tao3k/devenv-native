use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ComparisonOperator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParsedBooleanPathCondition<'a> {
    pub(super) negated: bool,
    pub(super) path: &'a str,
}

pub(super) fn parse_boolean_path_condition(source: &str) -> Option<ParsedBooleanPathCondition<'_>> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (negated, path) = match trimmed.strip_prefix("not ") {
        Some(path) => (true, path.trim()),
        None => (false, trimmed),
    };
    is_identifier_path(path).then_some(ParsedBooleanPathCondition { negated, path })
}

pub(super) fn parse_comparison_operator(source: &str) -> Option<ComparisonOperator> {
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

pub(super) fn parse_numeric_literal(source: &str) -> Option<f64> {
    let parsed = source.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}

pub(super) fn compare_numbers(lhs: f64, operator: ComparisonOperator, rhs: f64) -> bool {
    let ordering = lhs.total_cmp(&rhs);
    match operator {
        ComparisonOperator::Eq => ordering.is_eq(),
        ComparisonOperator::Ne => !ordering.is_eq(),
        ComparisonOperator::Lt => ordering.is_lt(),
        ComparisonOperator::Le => ordering.is_le(),
        ComparisonOperator::Gt => ordering.is_gt(),
        ComparisonOperator::Ge => ordering.is_ge(),
    }
}

pub(super) fn resolve_boolean_variable_path(variables: &Value, path: &str) -> Option<bool> {
    resolve_variable_path(variables, path)?.as_bool()
}

pub(super) fn resolve_numeric_variable_path(variables: &Value, path: &str) -> Option<f64> {
    resolve_variable_path(variables, path)?
        .as_f64()
        .filter(|value| value.is_finite())
}

pub(super) fn is_identifier_path(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(is_identifier_segment)
}

fn resolve_variable_path<'a>(variables: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = variables;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
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
