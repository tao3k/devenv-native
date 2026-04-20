//! DMN model and bounded single-decision contract types.

use serde_json::Value;
use std::sync::Arc;

/// Future DMN binding kind associated with one BPMN node or evaluation request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnBindingKind {
    /// A logical decision identifier reference.
    DecisionRef,
}

/// In-memory DMN source input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmnSourceFile {
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Raw XML or DMN content.
    pub contents: String,
}

impl DmnSourceFile {
    /// Creates a DMN source input.
    #[must_use]
    pub fn new(source_id: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            contents: contents.into(),
        }
    }
}

/// Placeholder link from BPMN to a future DMN decision artifact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionRef {
    /// Stable decision identifier.
    pub decision_id: Arc<str>,
    /// Optional source or namespace identifier.
    pub source_id: Option<Arc<str>>,
    /// Binding kind used for the reference.
    pub binding: DmnBindingKind,
}

impl DmnDecisionRef {
    /// Creates one decision reference placeholder.
    #[must_use]
    pub fn new(decision_id: impl AsRef<str>) -> Self {
        Self {
            decision_id: Arc::<str>::from(decision_id.as_ref()),
            source_id: None,
            binding: DmnBindingKind::DecisionRef,
        }
    }

    /// Adds an optional source identifier.
    #[must_use]
    pub fn with_source_id(mut self, source_id: impl AsRef<str>) -> Self {
        self.source_id = Some(Arc::<str>::from(source_id.as_ref()));
        self
    }
}

/// Supported bounded DMN hit policies.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnHitPolicy {
    /// Return the first matching rule output.
    Unique,
    /// Collect outputs from every matching rule into arrays.
    Collect,
}

/// One bounded DMN input clause.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInputClause {
    /// Stable input identifier.
    pub input_id: Arc<str>,
    /// Optional human-readable label.
    pub label: Option<Arc<str>>,
    /// Optional input name.
    pub name: Option<Arc<str>>,
    /// Optional input expression used to resolve variables.
    pub expression: Option<Arc<str>>,
}

impl DmnInputClause {
    /// Creates one bounded input clause.
    #[must_use]
    pub fn new(
        input_id: impl AsRef<str>,
        label: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
        expression: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            input_id: Arc::<str>::from(input_id.as_ref()),
            label: label.map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            expression: expression.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Returns the preferred variable lookup path for this input clause.
    #[must_use]
    pub fn lookup_path(&self) -> Option<&str> {
        self.expression
            .as_deref()
            .filter(|value| !value.is_empty())
            .or_else(|| self.name.as_deref().filter(|value| !value.is_empty()))
            .or_else(|| self.label.as_deref().filter(|value| !value.is_empty()))
    }
}

/// One bounded DMN output clause.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnOutputClause {
    /// Stable output identifier.
    pub output_id: Arc<str>,
    /// Optional human-readable label.
    pub label: Option<Arc<str>>,
    /// Optional output name.
    pub name: Option<Arc<str>>,
}

impl DmnOutputClause {
    /// Creates one bounded output clause.
    #[must_use]
    pub fn new(
        output_id: impl AsRef<str>,
        label: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            output_id: Arc::<str>::from(output_id.as_ref()),
            label: label.map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Returns the stable output key to use in evaluation results.
    #[must_use]
    pub fn output_key(&self) -> Arc<str> {
        self.name
            .as_ref()
            .filter(|value| !value.is_empty())
            .cloned()
            .or_else(|| {
                self.label
                    .as_ref()
                    .filter(|value| !value.is_empty())
                    .cloned()
            })
            .unwrap_or_else(|| Arc::clone(&self.output_id))
    }
}

/// One bounded DMN input-entry predicate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnInputEntry {
    /// Wildcard input that always matches.
    Any,
    /// Literal equality predicate.
    Equals(Value),
    /// Numeric comparison predicate such as `< 25` or `>= 25`.
    NumericComparison(DmnNumericComparison),
    /// Numeric range predicate such as `100 <= ? <= 110` or `[100..110]`.
    NumericRange(DmnNumericRange),
    /// ISO date comparison predicate such as `< date("2026-01-01")`.
    DateComparison(DmnDateComparison),
    /// ISO date range predicate such as
    /// `date("2026-01-01") <= ? < date("2026-01-31")`.
    DateRange(DmnDateRange),
}

/// Supported bounded DMN numeric comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnComparisonOperator {
    /// Strictly less-than comparison.
    LessThan,
    /// Less-than-or-equal comparison.
    LessThanOrEqual,
    /// Strictly greater-than comparison.
    GreaterThan,
    /// Greater-than-or-equal comparison.
    GreaterThanOrEqual,
}

/// One bounded numeric comparison predicate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnNumericComparison {
    /// Comparison operator.
    pub operator: DmnComparisonOperator,
    /// Numeric comparison target.
    pub value: f64,
}

impl DmnNumericComparison {
    /// Creates one bounded numeric comparison predicate.
    #[must_use]
    pub fn new(operator: DmnComparisonOperator, value: f64) -> Self {
        Self { operator, value }
    }
}

/// One numeric range bound within the bounded DMN subset.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnNumericRangeBound {
    /// Bound value.
    pub value: f64,
    /// Whether the bound is inclusive.
    pub inclusive: bool,
}

impl DmnNumericRangeBound {
    /// Creates one numeric range bound.
    #[must_use]
    pub fn new(value: f64, inclusive: bool) -> Self {
        Self { value, inclusive }
    }
}

/// One bounded numeric range predicate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnNumericRange {
    /// Optional lower bound.
    pub lower: Option<DmnNumericRangeBound>,
    /// Optional upper bound.
    pub upper: Option<DmnNumericRangeBound>,
}

impl DmnNumericRange {
    /// Creates one bounded numeric range predicate.
    #[must_use]
    pub fn new(lower: Option<DmnNumericRangeBound>, upper: Option<DmnNumericRangeBound>) -> Self {
        Self { lower, upper }
    }
}

/// One bounded ISO date comparison predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDateComparison {
    /// Comparison operator.
    pub operator: DmnComparisonOperator,
    /// ISO date comparison target in `YYYY-MM-DD` form.
    pub value: Arc<str>,
}

impl DmnDateComparison {
    /// Creates one bounded ISO date comparison predicate.
    #[must_use]
    pub fn new(operator: DmnComparisonOperator, value: impl AsRef<str>) -> Self {
        Self {
            operator,
            value: Arc::<str>::from(value.as_ref()),
        }
    }
}

/// One ISO date range bound within the bounded DMN subset.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDateRangeBound {
    /// ISO date bound value in `YYYY-MM-DD` form.
    pub value: Arc<str>,
    /// Whether the bound is inclusive.
    pub inclusive: bool,
}

impl DmnDateRangeBound {
    /// Creates one ISO date range bound.
    #[must_use]
    pub fn new(value: impl AsRef<str>, inclusive: bool) -> Self {
        Self {
            value: Arc::<str>::from(value.as_ref()),
            inclusive,
        }
    }
}

/// One bounded ISO date range predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDateRange {
    /// Optional lower bound.
    pub lower: Option<DmnDateRangeBound>,
    /// Optional upper bound.
    pub upper: Option<DmnDateRangeBound>,
}

impl DmnDateRange {
    /// Creates one bounded ISO date range predicate.
    #[must_use]
    pub fn new(lower: Option<DmnDateRangeBound>, upper: Option<DmnDateRangeBound>) -> Self {
        Self { lower, upper }
    }
}

/// One bounded DMN output-entry expression.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnOutputEntry {
    /// Literal output value.
    pub value: Value,
}

impl DmnOutputEntry {
    /// Creates one bounded output entry.
    #[must_use]
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

/// One bounded DMN rule.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnRule {
    /// Stable rule identifier.
    pub rule_id: Arc<str>,
    /// Optional free-form rule description.
    pub description: Option<Arc<str>>,
    /// Input predicates for this rule.
    pub input_entries: Vec<DmnInputEntry>,
    /// Output expressions for this rule.
    pub output_entries: Vec<DmnOutputEntry>,
}

impl DmnRule {
    /// Creates one bounded DMN rule.
    #[must_use]
    pub fn new(
        rule_id: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
        input_entries: Vec<DmnInputEntry>,
        output_entries: Vec<DmnOutputEntry>,
    ) -> Self {
        Self {
            rule_id: Arc::<str>::from(rule_id.as_ref()),
            description: description.map(|value| Arc::<str>::from(value.as_ref())),
            input_entries,
            output_entries,
        }
    }
}

/// One bounded DMN decision table.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionTable {
    /// Stable decision-table identifier.
    pub table_id: Arc<str>,
    /// Optional table name.
    pub name: Option<Arc<str>>,
    /// Table hit policy.
    pub hit_policy: DmnHitPolicy,
    /// Ordered input clauses.
    pub inputs: Vec<DmnInputClause>,
    /// Ordered output clauses.
    pub outputs: Vec<DmnOutputClause>,
    /// Ordered rules.
    pub rules: Vec<DmnRule>,
}

impl DmnDecisionTable {
    /// Creates one bounded decision table.
    #[must_use]
    pub fn new(
        table_id: impl AsRef<str>,
        name: Option<impl AsRef<str>>,
        hit_policy: DmnHitPolicy,
        inputs: Vec<DmnInputClause>,
        outputs: Vec<DmnOutputClause>,
        rules: Vec<DmnRule>,
    ) -> Self {
        Self {
            table_id: Arc::<str>::from(table_id.as_ref()),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            hit_policy,
            inputs,
            outputs,
            rules,
        }
    }
}

/// One bounded DMN decision definition with one decision table.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionDefinition {
    /// Source identifier used for diagnostics.
    pub source_id: Arc<str>,
    /// Stable decision reference.
    pub decision: DmnDecisionRef,
    /// Optional decision name.
    pub name: Option<Arc<str>>,
    /// The single bounded decision table.
    pub table: DmnDecisionTable,
}

impl DmnDecisionDefinition {
    /// Creates one bounded decision definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        decision: DmnDecisionRef,
        name: Option<impl AsRef<str>>,
        table: DmnDecisionTable,
    ) -> Self {
        Self {
            source_id: Arc::<str>::from(source_id.as_ref()),
            decision,
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            table,
        }
    }

    /// Returns whether the provided reference matches this parsed decision.
    #[must_use]
    pub fn matches_reference(&self, other: &DmnDecisionRef) -> bool {
        self.decision.decision_id == other.decision_id
            && other
                .source_id
                .as_deref()
                .is_none_or(|source_id| source_id == self.source_id.as_ref())
    }
}

/// DMN evaluation request surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnEvaluationRequest {
    /// Target decision reference.
    pub decision: DmnDecisionRef,
    /// Input variables supplied by the host.
    pub variables: serde_json::Value,
}

impl DmnEvaluationRequest {
    /// Creates one DMN evaluation request.
    #[must_use]
    pub fn new(decision: DmnDecisionRef, variables: serde_json::Value) -> Self {
        Self {
            decision,
            variables,
        }
    }
}

/// DMN evaluation result surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnEvaluationResult {
    /// Evaluated decision identity.
    pub decision_id: Arc<str>,
    /// Output payload.
    pub output: Value,
    /// Rule identifiers that matched during evaluation.
    pub matched_rule_ids: Vec<Arc<str>>,
}

impl DmnEvaluationResult {
    /// Creates one DMN evaluation result.
    #[must_use]
    pub fn new(
        decision_id: impl AsRef<str>,
        output: Value,
        matched_rule_ids: Vec<Arc<str>>,
    ) -> Self {
        Self {
            decision_id: Arc::<str>::from(decision_id.as_ref()),
            output,
            matched_rule_ids,
        }
    }
}
