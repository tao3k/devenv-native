//! Bounded DMN evaluation internals.

use crate::dmn_model_api::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateTimeComparison,
    DmnDateTimeRange, DmnDecisionDefinition, DmnEvaluationRequest, DmnEvaluationResult,
    DmnHitPolicy, DmnInputClause, DmnInputEntry, DmnNumericRange, DmnRule, DmnTimeComparison,
    DmnTimeRange,
};
use crate::error::{BpmnEngineError, Result};
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use serde_json::{Map, Value};
use std::sync::Arc;

/// Synchronous bounded DMN evaluation entrypoint for in-engine runtime paths.
pub(crate) fn evaluate_dmn_decision_sync(
    decision: &DmnDecisionDefinition,
    request: &DmnEvaluationRequest,
) -> Result<DmnEvaluationResult> {
    if !decision.matches_reference(&request.decision) {
        return Err(BpmnEngineError::DmnDecisionMismatch {
            expected: decision.decision.decision_id.to_string(),
            actual: request.decision.decision_id.to_string(),
        });
    }

    let mut matched_rule_ids = Vec::new();
    match decision.table.hit_policy {
        DmnHitPolicy::Unique => {
            for rule in &decision.table.rules {
                if rule_matches(decision, rule, &request.variables) {
                    matched_rule_ids.push(Arc::clone(&rule.rule_id));
                    return Ok(DmnEvaluationResult::new(
                        decision.decision.decision_id.as_ref(),
                        unique_rule_output(decision, rule),
                        matched_rule_ids,
                    ));
                }
            }
            Ok(DmnEvaluationResult::new(
                decision.decision.decision_id.as_ref(),
                Value::Object(Map::new()),
                matched_rule_ids,
            ))
        }
        DmnHitPolicy::Collect => {
            let mut output = Map::new();
            for rule in &decision.table.rules {
                if !rule_matches(decision, rule, &request.variables) {
                    continue;
                }
                matched_rule_ids.push(Arc::clone(&rule.rule_id));
                for (output_clause, output_entry) in
                    decision.table.outputs.iter().zip(&rule.output_entries)
                {
                    let key = output_clause.output_key();
                    let slot = output
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Array(Vec::new()));
                    if let Value::Array(values) = slot {
                        values.push(output_entry.value.clone());
                    }
                }
            }
            Ok(DmnEvaluationResult::new(
                decision.decision.decision_id.as_ref(),
                Value::Object(output),
                matched_rule_ids,
            ))
        }
    }
}

fn rule_matches(decision: &DmnDecisionDefinition, rule: &DmnRule, variables: &Value) -> bool {
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
            DmnInputEntry::NumericComparison(comparison) => evaluate_numeric_comparison(
                &resolve_input_value(variables, input_clause),
                comparison.operator,
                comparison.value,
            ),
            DmnInputEntry::NumericRange(range) => {
                evaluate_numeric_range(&resolve_input_value(variables, input_clause), range)
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

fn unique_rule_output(decision: &DmnDecisionDefinition, rule: &DmnRule) -> Value {
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

fn evaluate_date_time_comparison(actual: &Value, comparison: &DmnDateTimeComparison) -> bool {
    let Some(actual) = parse_iso_datetime_value(actual) else {
        return false;
    };
    let Some(expected) = parse_iso_datetime_str(&comparison.value) else {
        return false;
    };
    match comparison.operator {
        DmnComparisonOperator::LessThan => actual < expected,
        DmnComparisonOperator::LessThanOrEqual => actual <= expected,
        DmnComparisonOperator::GreaterThan => actual > expected,
        DmnComparisonOperator::GreaterThanOrEqual => actual >= expected,
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
        if (lower.inclusive && actual < lower_value) || (!lower.inclusive && actual <= lower_value)
        {
            return false;
        }
    }
    if let Some(upper) = &range.upper {
        let Some(upper_value) = parse_iso_datetime_str(&upper.value) else {
            return false;
        };
        if (upper.inclusive && actual > upper_value) || (!upper.inclusive && actual >= upper_value)
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

fn parse_iso_datetime_value(value: &Value) -> Option<NaiveDateTime> {
    let Value::String(value) = value else {
        return None;
    };
    parse_iso_datetime_str(value)
}

fn parse_iso_datetime_str(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").ok()
}
