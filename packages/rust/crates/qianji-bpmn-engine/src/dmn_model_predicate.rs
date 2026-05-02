//! Public dmn model predicate contracts for BPMN/DMN engine integration.

use serde_json::Value;
use std::sync::Arc;

/// One bounded DMN input-entry predicate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnInputEntry {
    /// Wildcard input that always matches.
    Any,
    /// Literal equality predicate.
    Equals(Value),
    /// Positive ISO 8601 bounded duration literal equality predicate.
    DurationEquals(Arc<str>),
    /// ISO local or offset-aware datetime literal equality predicate.
    DateTimeEquals(Arc<str>),
    /// Numeric comparison predicate such as `< 25` or `>= 25`.
    NumericComparison(DmnNumericComparison),
    /// Positive ISO 8601 bounded duration comparison predicate.
    DurationComparison(DmnDurationComparison),
    /// Numeric range predicate such as `100 <= ? <= 110` or `[100..110]`.
    NumericRange(DmnNumericRange),
    /// Positive ISO 8601 bounded duration range predicate.
    DurationRange(DmnDurationRange),
    /// ISO date comparison predicate such as `< date("2026-01-01")`.
    DateComparison(DmnDateComparison),
    /// ISO date range predicate such as
    /// `date("2026-01-01") <= ? < date("2026-01-31")`.
    DateRange(DmnDateRange),
    /// ISO local datetime comparison predicate such as
    /// `< date and time("2026-01-01T09:00:00")`.
    DateTimeComparison(DmnDateTimeComparison),
    /// ISO local datetime range predicate such as
    /// `date and time("2026-01-01T09:00:00") <= ? < date and time("2026-01-01T17:00:00")`.
    DateTimeRange(DmnDateTimeRange),
    /// ISO time comparison predicate such as `< time("09:00:00")`.
    TimeComparison(DmnTimeComparison),
    /// ISO time range predicate such as
    /// `time("09:00:00") <= ? < time("17:00:00")`.
    TimeRange(DmnTimeRange),
}

/// Supported bounded DMN comparison operators.
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

/// One bounded ISO 8601 duration comparison predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDurationComparison {
    /// Comparison operator.
    pub operator: DmnComparisonOperator,
    /// Duration comparison target in bounded `duration("...")` form.
    pub value: Arc<str>,
}

impl DmnDurationComparison {
    /// Creates one bounded duration comparison predicate.
    #[must_use]
    pub fn new(operator: DmnComparisonOperator, value: impl AsRef<str>) -> Self {
        Self {
            operator,
            value: Arc::<str>::from(value.as_ref()),
        }
    }
}

/// One bounded ISO 8601 duration range bound.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDurationRangeBound {
    /// Duration bound value in bounded `duration("...")` form.
    pub value: Arc<str>,
    /// Whether the bound is inclusive.
    pub inclusive: bool,
}

impl DmnDurationRangeBound {
    /// Creates one bounded duration range bound.
    #[must_use]
    pub fn new(value: impl AsRef<str>, inclusive: bool) -> Self {
        Self {
            value: Arc::<str>::from(value.as_ref()),
            inclusive,
        }
    }
}

/// One bounded ISO 8601 duration range predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDurationRange {
    /// Optional lower bound.
    pub lower: Option<DmnDurationRangeBound>,
    /// Optional upper bound.
    pub upper: Option<DmnDurationRangeBound>,
}

impl DmnDurationRange {
    /// Creates one bounded duration range predicate.
    #[must_use]
    pub fn new(lower: Option<DmnDurationRangeBound>, upper: Option<DmnDurationRangeBound>) -> Self {
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

/// One bounded ISO local datetime comparison predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDateTimeComparison {
    /// Comparison operator.
    pub operator: DmnComparisonOperator,
    /// ISO local or offset-aware datetime comparison target in
    /// `YYYY-MM-DDTHH:MM:SS` or RFC3339 form.
    pub value: Arc<str>,
}

impl DmnDateTimeComparison {
    /// Creates one bounded ISO local datetime comparison predicate.
    #[must_use]
    pub fn new(operator: DmnComparisonOperator, value: impl AsRef<str>) -> Self {
        Self {
            operator,
            value: Arc::<str>::from(value.as_ref()),
        }
    }
}

/// One ISO local datetime range bound within the bounded DMN subset.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDateTimeRangeBound {
    /// ISO local or offset-aware datetime bound value in
    /// `YYYY-MM-DDTHH:MM:SS` or RFC3339 form.
    pub value: Arc<str>,
    /// Whether the bound is inclusive.
    pub inclusive: bool,
}

impl DmnDateTimeRangeBound {
    /// Creates one ISO local datetime range bound.
    #[must_use]
    pub fn new(value: impl AsRef<str>, inclusive: bool) -> Self {
        Self {
            value: Arc::<str>::from(value.as_ref()),
            inclusive,
        }
    }
}

/// One bounded ISO local datetime range predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDateTimeRange {
    /// Optional lower bound.
    pub lower: Option<DmnDateTimeRangeBound>,
    /// Optional upper bound.
    pub upper: Option<DmnDateTimeRangeBound>,
}

impl DmnDateTimeRange {
    /// Creates one bounded ISO local datetime range predicate.
    #[must_use]
    pub fn new(lower: Option<DmnDateTimeRangeBound>, upper: Option<DmnDateTimeRangeBound>) -> Self {
        Self { lower, upper }
    }
}

/// One bounded ISO time comparison predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnTimeComparison {
    /// Comparison operator.
    pub operator: DmnComparisonOperator,
    /// ISO time comparison target in `HH:MM:SS` form.
    pub value: Arc<str>,
}

impl DmnTimeComparison {
    /// Creates one bounded ISO time comparison predicate.
    #[must_use]
    pub fn new(operator: DmnComparisonOperator, value: impl AsRef<str>) -> Self {
        Self {
            operator,
            value: Arc::<str>::from(value.as_ref()),
        }
    }
}

/// One ISO time range bound within the bounded DMN subset.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnTimeRangeBound {
    /// ISO time bound value in `HH:MM:SS` form.
    pub value: Arc<str>,
    /// Whether the bound is inclusive.
    pub inclusive: bool,
}

impl DmnTimeRangeBound {
    /// Creates one ISO time range bound.
    #[must_use]
    pub fn new(value: impl AsRef<str>, inclusive: bool) -> Self {
        Self {
            value: Arc::<str>::from(value.as_ref()),
            inclusive,
        }
    }
}

/// One bounded ISO time range predicate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnTimeRange {
    /// Optional lower bound.
    pub lower: Option<DmnTimeRangeBound>,
    /// Optional upper bound.
    pub upper: Option<DmnTimeRangeBound>,
}

impl DmnTimeRange {
    /// Creates one bounded ISO time range predicate.
    #[must_use]
    pub fn new(lower: Option<DmnTimeRangeBound>, upper: Option<DmnTimeRangeBound>) -> Self {
        Self { lower, upper }
    }
}
