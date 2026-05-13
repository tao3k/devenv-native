//! JSON filter conversion for Lance SQL-like WHERE clauses.

/// Convert JSON filter expression to `LanceDB` WHERE clause.
#[must_use]
pub fn json_to_lance_where(expr: &serde_json::Value) -> String {
    let serde_json::Value::Object(object) = expr else {
        return String::new();
    };
    object
        .iter()
        .filter_map(|(key, value)| lance_clause_for_value(key, value))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn lance_clause_for_value(key: &str, value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(comparison) => lance_comparison_clause(key, comparison),
        serde_json::Value::String(value) => Some(format!("{key} = '{value}'")),
        serde_json::Value::Number(value) => Some(format!("{key} = {value}")),
        serde_json::Value::Bool(value) => Some(format!("{key} = {value}")),
        _ => None,
    }
}

fn lance_comparison_clause(
    key: &str,
    comparison: &serde_json::Map<String, serde_json::Value>,
) -> Option<String> {
    comparison
        .iter()
        .find_map(|(operator, value)| comparison_operator(operator).map(|op| (op, value)))
        .map(|(operator, value)| format!("{key} {operator} {}", lance_literal(value)))
}

fn comparison_operator(operator: &str) -> Option<&'static str> {
    match operator {
        "$gt" | ">" => Some(">"),
        "$gte" | ">=" => Some(">="),
        "$lt" | "<" => Some("<"),
        "$lte" | "<=" => Some("<="),
        "$ne" | "!=" => Some("!="),
        _ => None,
    }
}

fn lance_literal(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => format!("'{value}'"),
        _ => value.to_string(),
    }
}
