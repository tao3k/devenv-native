use crate::dmn_duration::{
    DmnDurationValue, parse_day_time_duration_str, parse_year_month_duration_str,
};
use crate::dmn_model_api::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateTimeComparison,
    DmnDateTimeRange, DmnDecisionDefinition, DmnDurationComparison, DmnDurationRange,
    DmnInputClause, DmnInputEntry, DmnNumericRange, DmnRule, DmnTimeComparison, DmnTimeRange,
};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::{Map, Value};
use std::cmp::Ordering;

pub(super) fn rule_matches(
    decision: &DmnDecisionDefinition,
    rule: &DmnRule,
    variables: &Value,
) -> bool {
    decision
        .table
        .inputs
        .iter()
        .zip(&rule.input_entries)
        .all(|(input_clause, input_entry)| match input_entry {
            DmnInputEntry::Any => true,
            DmnInputEntry::Equals(expected) => {
                resolve_input_value(variables, input_clause) == *expected
            }
            DmnInputEntry::DurationEquals(expected) => {
                evaluate_duration_equals(&resolve_input_value(variables, input_clause), expected)
            }
            DmnInputEntry::DateTimeEquals(expected) => {
                evaluate_date_time_equals(&resolve_input_value(variables, input_clause), expected)
            }
            DmnInputEntry::NumericComparison(comparison) => evaluate_numeric_comparison(
                &resolve_input_value(variables, input_clause),
                comparison.operator,
                comparison.value,
            ),
            DmnInputEntry::DurationComparison(comparison) => evaluate_duration_comparison(
                &resolve_input_value(variables, input_clause),
                comparison,
            ),
            DmnInputEntry::NumericRange(range) => {
                evaluate_numeric_range(&resolve_input_value(variables, input_clause), range)
            }
            DmnInputEntry::DurationRange(range) => {
                evaluate_duration_range(&resolve_input_value(variables, input_clause), range)
            }
            DmnInputEntry::DateComparison(comparison) => {
                evaluate_date_comparison(&resolve_input_value(variables, input_clause), comparison)
            }
            DmnInputEntry::DateRange(range) => {
                evaluate_date_range(&resolve_input_value(variables, input_clause), range)
            }
            DmnInputEntry::DateTimeComparison(comparison) => evaluate_date_time_comparison(
                &resolve_input_value(variables, input_clause),
                comparison,
            ),
            DmnInputEntry::DateTimeRange(range) => {
                evaluate_date_time_range(&resolve_input_value(variables, input_clause), range)
            }
            DmnInputEntry::TimeComparison(comparison) => {
                evaluate_time_comparison(&resolve_input_value(variables, input_clause), comparison)
            }
            DmnInputEntry::TimeRange(range) => {
                evaluate_time_range(&resolve_input_value(variables, input_clause), range)
            }
        })
}

fn resolve_input_value(variables: &Value, input_clause: &DmnInputClause) -> Value {
    let Some(path) = input_clause.lookup_path() else {
        return Value::Null;
    };

    let mut current = variables;
    for segment in path.split('.') {
        let Some(next) = current.get(segment) else {
            return Value::Null;
        };
        current = next;
    }
    current.clone()
}

pub(super) fn unique_rule_output(decision: &DmnDecisionDefinition, rule: &DmnRule) -> Value {
    let mut output = Map::new();
    for (output_clause, output_entry) in decision.table.outputs.iter().zip(&rule.output_entries) {
        output.insert(
            output_clause.output_key().to_string(),
            output_entry.value.clone(),
        );
    }
    Value::Object(output)
}

fn evaluate_numeric_comparison(
    actual: &Value,
    operator: DmnComparisonOperator,
    expected: f64,
) -> bool {
    let Some(actual) = actual.as_f64() else {
        return false;
    };
    match operator {
        DmnComparisonOperator::LessThan => actual < expected,
        DmnComparisonOperator::LessThanOrEqual => actual <= expected,
        DmnComparisonOperator::GreaterThan => actual > expected,
        DmnComparisonOperator::GreaterThanOrEqual => actual >= expected,
    }
}

fn evaluate_duration_equals(actual: &Value, expected: &str) -> bool {
    let Some(actual) = parse_duration_value(actual) else {
        return false;
    };
    let Some(expected) = parse_duration_str(expected) else {
        return false;
    };
    actual == expected
}

fn evaluate_duration_comparison(actual: &Value, comparison: &DmnDurationComparison) -> bool {
    let Some(actual) = parse_duration_value(actual) else {
        return false;
    };
    let Some(expected) = parse_duration_str(&comparison.value) else {
        return false;
    };
    let Some(ordering) = actual.compare(expected) else {
        return false;
    };
    match comparison.operator {
        DmnComparisonOperator::LessThan => ordering == Ordering::Less,
        DmnComparisonOperator::LessThanOrEqual => {
            matches!(ordering, Ordering::Less | Ordering::Equal)
        }
        DmnComparisonOperator::GreaterThan => ordering == Ordering::Greater,
        DmnComparisonOperator::GreaterThanOrEqual => {
            matches!(ordering, Ordering::Greater | Ordering::Equal)
        }
    }
}

fn evaluate_numeric_range(actual: &Value, range: &DmnNumericRange) -> bool {
    let Some(actual) = actual.as_f64() else {
        return false;
    };
    if let Some(lower) = &range.lower
        && ((lower.inclusive && actual < lower.value)
            || (!lower.inclusive && actual <= lower.value))
    {
        return false;
    }
    if let Some(upper) = &range.upper
        && ((upper.inclusive && actual > upper.value)
            || (!upper.inclusive && actual >= upper.value))
    {
        return false;
    }
    true
}

fn evaluate_duration_range(actual: &Value, range: &DmnDurationRange) -> bool {
    let Some(actual) = parse_duration_value(actual) else {
        return false;
    };
    if let Some(lower) = &range.lower {
        let Some(lower_value) = parse_duration_str(&lower.value) else {
            return false;
        };
        let Some(ordering) = actual.compare(lower_value) else {
            return false;
        };
        if (lower.inclusive && ordering == Ordering::Less)
            || (!lower.inclusive && matches!(ordering, Ordering::Less | Ordering::Equal))
        {
            return false;
        }
    }
    if let Some(upper) = &range.upper {
        let Some(upper_value) = parse_duration_str(&upper.value) else {
            return false;
        };
        let Some(ordering) = actual.compare(upper_value) else {
            return false;
        };
        if (upper.inclusive && ordering == Ordering::Greater)
            || (!upper.inclusive && matches!(ordering, Ordering::Greater | Ordering::Equal))
        {
            return false;
        }
    }
    true
}

fn evaluate_date_comparison(actual: &Value, comparison: &DmnDateComparison) -> bool {
    let Some(actual) = parse_iso_date_value(actual) else {
        return false;
    };
    let Some(expected) = parse_iso_date_str(&comparison.value) else {
        return false;
    };
    match comparison.operator {
        DmnComparisonOperator::LessThan => actual < expected,
        DmnComparisonOperator::LessThanOrEqual => actual <= expected,
        DmnComparisonOperator::GreaterThan => actual > expected,
        DmnComparisonOperator::GreaterThanOrEqual => actual >= expected,
    }
}

fn evaluate_date_range(actual: &Value, range: &DmnDateRange) -> bool {
    let Some(actual) = parse_iso_date_value(actual) else {
        return false;
    };
    if let Some(lower) = &range.lower {
        let Some(lower_value) = parse_iso_date_str(&lower.value) else {
            return false;
        };
        if (lower.inclusive && actual < lower_value) || (!lower.inclusive && actual <= lower_value)
        {
            return false;
        }
    }
    if let Some(upper) = &range.upper {
        let Some(upper_value) = parse_iso_date_str(&upper.value) else {
            return false;
        };
        if (upper.inclusive && actual > upper_value) || (!upper.inclusive && actual >= upper_value)
        {
            return false;
        }
    }
    true
}

fn evaluate_time_comparison(actual: &Value, comparison: &DmnTimeComparison) -> bool {
    let Some(actual) = parse_iso_time_value(actual) else {
        return false;
    };
    let Some(expected) = parse_iso_time_str(&comparison.value) else {
        return false;
    };
    match comparison.operator {
        DmnComparisonOperator::LessThan => actual < expected,
        DmnComparisonOperator::LessThanOrEqual => actual <= expected,
        DmnComparisonOperator::GreaterThan => actual > expected,
        DmnComparisonOperator::GreaterThanOrEqual => actual >= expected,
    }
}

fn evaluate_date_time_equals(actual: &Value, expected: &str) -> bool {
    let Some(actual) = parse_iso_datetime_value(actual) else {
        return false;
    };
    let Some(expected) = parse_iso_datetime_str(expected) else {
        return false;
    };
    compare_date_time_values(&actual, &expected) == Ordering::Equal
}

fn evaluate_date_time_comparison(actual: &Value, comparison: &DmnDateTimeComparison) -> bool {
    let Some(actual) = parse_iso_datetime_value(actual) else {
        return false;
    };
    let Some(expected) = parse_iso_datetime_str(&comparison.value) else {
        return false;
    };
    let ordering = compare_date_time_values(&actual, &expected);
    match comparison.operator {
        DmnComparisonOperator::LessThan => ordering == Ordering::Less,
        DmnComparisonOperator::LessThanOrEqual => {
            matches!(ordering, Ordering::Less | Ordering::Equal)
        }
        DmnComparisonOperator::GreaterThan => ordering == Ordering::Greater,
        DmnComparisonOperator::GreaterThanOrEqual => {
            matches!(ordering, Ordering::Greater | Ordering::Equal)
        }
    }
}

fn evaluate_date_time_range(actual: &Value, range: &DmnDateTimeRange) -> bool {
    let Some(actual) = parse_iso_datetime_value(actual) else {
        return false;
    };
    if let Some(lower) = &range.lower {
        let Some(lower_value) = parse_iso_datetime_str(&lower.value) else {
            return false;
        };
        let ordering = compare_date_time_values(&actual, &lower_value);
        if (lower.inclusive && ordering == Ordering::Less)
            || (!lower.inclusive && matches!(ordering, Ordering::Less | Ordering::Equal))
        {
            return false;
        }
    }
    if let Some(upper) = &range.upper {
        let Some(upper_value) = parse_iso_datetime_str(&upper.value) else {
            return false;
        };
        let ordering = compare_date_time_values(&actual, &upper_value);
        if (upper.inclusive && ordering == Ordering::Greater)
            || (!upper.inclusive && matches!(ordering, Ordering::Greater | Ordering::Equal))
        {
            return false;
        }
    }
    true
}

fn evaluate_time_range(actual: &Value, range: &DmnTimeRange) -> bool {
    let Some(actual) = parse_iso_time_value(actual) else {
        return false;
    };
    if let Some(lower) = &range.lower {
        let Some(lower_value) = parse_iso_time_str(&lower.value) else {
            return false;
        };
        if (lower.inclusive && actual < lower_value) || (!lower.inclusive && actual <= lower_value)
        {
            return false;
        }
    }
    if let Some(upper) = &range.upper {
        let Some(upper_value) = parse_iso_time_str(&upper.value) else {
            return false;
        };
        if (upper.inclusive && actual > upper_value) || (!upper.inclusive && actual >= upper_value)
        {
            return false;
        }
    }
    true
}

fn parse_iso_date_value(value: &Value) -> Option<NaiveDate> {
    let Value::String(value) = value else {
        return None;
    };
    parse_iso_date_str(value)
}

fn parse_duration_value(value: &Value) -> Option<DmnDurationValue> {
    let Value::String(value) = value else {
        return None;
    };
    parse_duration_str(value)
}

fn parse_duration_str(value: &str) -> Option<DmnDurationValue> {
    parse_day_time_duration_str(value).or_else(|| parse_year_month_duration_str(value))
}

fn parse_iso_date_str(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn parse_iso_time_value(value: &Value) -> Option<NaiveTime> {
    let Value::String(value) = value else {
        return None;
    };
    parse_iso_time_str(value)
}

fn parse_iso_time_str(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M:%S").ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DmnComparableDateTime {
    Local(NaiveDateTime),
    Offset(DateTime<FixedOffset>),
}

fn parse_iso_datetime_value(value: &Value) -> Option<DmnComparableDateTime> {
    let Value::String(value) = value else {
        return None;
    };
    parse_iso_datetime_str(value)
}

fn parse_iso_datetime_str(value: &str) -> Option<DmnComparableDateTime> {
    if let Ok(value) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return Some(DmnComparableDateTime::Local(value));
    }
    DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .ok()
        .map(DmnComparableDateTime::Offset)
}

fn compare_date_time_values(
    left: &DmnComparableDateTime,
    right: &DmnComparableDateTime,
) -> Ordering {
    date_time_utc(left).cmp(&date_time_utc(right))
}

fn date_time_utc(value: &DmnComparableDateTime) -> DateTime<Utc> {
    match value {
        // Bounded mixed-form coercion rule: local datetimes are interpreted as
        // UTC instants whenever they need to compare against offset-aware
        // datetimes.
        DmnComparableDateTime::Local(value) => value.and_utc(),
        DmnComparableDateTime::Offset(value) => value.with_timezone(&Utc),
    }
}
