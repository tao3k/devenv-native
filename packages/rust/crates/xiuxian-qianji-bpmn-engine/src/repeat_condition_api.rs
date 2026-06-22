//! Public repeat-condition parse summary surface.

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

/// Parses one bounded exclusive-gateway condition into a structured summary.
#[must_use]
pub fn parse_gateway_condition_summary(condition: &str) -> Option<GatewayConditionSummary> {
    crate::repeat_condition::parse_gateway_condition_summary(condition)
}
