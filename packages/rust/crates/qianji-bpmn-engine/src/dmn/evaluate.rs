//! Bounded DMN evaluation internals.

use crate::dmn::{
    evaluate_dmn_context_expression_decision, evaluate_dmn_list_expression_decision,
    evaluate_dmn_literal_expression_decision, evaluate_dmn_relation_expression_decision,
};
use crate::dmn_duration::{
    DmnDurationValue, parse_day_time_duration_str, parse_year_month_duration_str,
};
use crate::dmn_model_api::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateTimeComparison,
    DmnDateTimeRange, DmnDecisionDefinition, DmnDecisionRef, DmnDurationComparison,
    DmnDurationRange, DmnEvaluationRequest, DmnEvaluationResult, DmnHitPolicy,
    DmnInformationRequirementReference, DmnInputClause, DmnInputEntry, DmnNumericRange, DmnRule,
    DmnTimeComparison, DmnTimeRange,
};
use crate::error::{BpmnEngineError, Result};
use crate::ir::BpmnPackage;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::sync::Arc;

/// Synchronous bounded package-aware DMN evaluation entrypoint for in-engine
/// runtime paths that need direct local required-decision resolution.
pub(crate) fn evaluate_dmn_package_decision_sync(
    package: &BpmnPackage,
    decision: &DmnDecisionDefinition,
    request: &DmnEvaluationRequest,
) -> Result<DmnEvaluationResult> {
    let mut stack = Vec::new();
    evaluate_dmn_package_decision_with_stack(package, decision, request, &mut stack)
}

/// Synchronous bounded DMN evaluation entrypoint for in-engine runtime paths.
pub(crate) fn evaluate_dmn_decision_sync(
    decision: &DmnDecisionDefinition,
    request: &DmnEvaluationRequest,
) -> Result<DmnEvaluationResult> {
    validate_request_matches_decision(decision, request)?;
    if let Some(literal) = decision.literal_expression.as_ref() {
        return evaluate_dmn_literal_expression_decision(decision, literal, &request.variables);
    }
    if let Some(list) = decision.list_expression.as_ref() {
        return evaluate_dmn_list_expression_decision(decision, list, &request.variables);
    }
    if let Some(context) = decision.context_expression.as_ref() {
        return evaluate_dmn_context_expression_decision(decision, context, &request.variables);
    }
    if let Some(relation) = decision.relation_expression.as_ref() {
        return evaluate_dmn_relation_expression_decision(decision, relation, &request.variables);
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

fn evaluate_dmn_package_decision_with_stack(
    package: &BpmnPackage,
    decision: &DmnDecisionDefinition,
    request: &DmnEvaluationRequest,
    stack: &mut Vec<(String, String)>,
) -> Result<DmnEvaluationResult> {
    validate_request_matches_decision(decision, request)?;
    let stack_key = decision_stack_key(decision);
    if stack.contains(&stack_key) {
        return Err(BpmnEngineError::CyclicDmnRequiredDecisionDependency {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
        });
    }

    stack.push(stack_key);
    let result = (|| {
        let variables = resolve_information_requirement_variables(
            package,
            decision,
            &request.variables,
            stack,
        )?;
        evaluate_dmn_decision_sync(
            decision,
            &DmnEvaluationRequest::new(request.decision.clone(), variables),
        )
    })();
    let _ = stack.pop();
    result
}

fn validate_request_matches_decision(
    decision: &DmnDecisionDefinition,
    request: &DmnEvaluationRequest,
) -> Result<()> {
    if decision.matches_reference(&request.decision) {
        return Ok(());
    }
    Err(BpmnEngineError::DmnDecisionMismatch {
        expected: decision.decision.decision_id.to_string(),
        actual: request.decision.decision_id.to_string(),
    })
}

fn decision_stack_key(decision: &DmnDecisionDefinition) -> (String, String) {
    (
        decision.source_id.to_string(),
        decision.decision.decision_id.to_string(),
    )
}

fn resolve_information_requirement_variables(
    package: &BpmnPackage,
    decision: &DmnDecisionDefinition,
    variables: &Value,
    stack: &mut Vec<(String, String)>,
) -> Result<Value> {
    let mut resolved = variables.clone();
    for requirement in &decision.information_requirements {
        match requirement.reference_kind.as_ref() {
            "requiredInput" => {
                bind_required_input_alias(package, decision, requirement, &mut resolved)?;
            }
            "requiredDecision" => {
                let dependency =
                    resolve_required_decision_definition(package, decision, requirement)?;
                let evaluation = evaluate_dmn_package_decision_with_stack(
                    package,
                    dependency,
                    &DmnEvaluationRequest::new(dependency.decision.clone(), resolved.clone()),
                    stack,
                )?;
                merge_evaluation_output(&mut resolved, &evaluation.output);
            }
            _ => {}
        }
    }
    Ok(resolved)
}

fn bind_required_input_alias(
    package: &BpmnPackage,
    decision: &DmnDecisionDefinition,
    requirement: &DmnInformationRequirementReference,
    variables: &mut Value,
) -> Result<()> {
    let href = information_requirement_href(decision, requirement)?;
    let Some(input_data) = package.find_dmn_input_data(decision.source_id.as_ref(), &href) else {
        return Err(BpmnEngineError::MissingDmnRequiredInputTarget {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
            href: requirement
                .href
                .as_deref()
                .unwrap_or("<missing>")
                .to_string(),
        });
    };
    let (Some(variable_name), Some(input_name)) = (
        input_data.variable_name.as_deref(),
        input_data.name.as_deref(),
    ) else {
        return Ok(());
    };
    let Some(object) = variables.as_object_mut() else {
        return Ok(());
    };
    if object.contains_key(variable_name) {
        return Ok(());
    }
    let Some(value) = object.get(input_name).cloned() else {
        return Ok(());
    };
    object.insert(variable_name.to_string(), value);
    Ok(())
}

fn resolve_required_decision_definition<'a>(
    package: &'a BpmnPackage,
    decision: &DmnDecisionDefinition,
    requirement: &DmnInformationRequirementReference,
) -> Result<&'a DmnDecisionDefinition> {
    let target_id = information_requirement_href(decision, requirement)?;
    let decision_ref = DmnDecisionRef::new(target_id).with_source_id(decision.source_id.as_ref());
    package.find_dmn_decision(&decision_ref)?.ok_or_else(|| {
        BpmnEngineError::MissingDmnRequiredDecisionTarget {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
            href: requirement
                .href
                .as_deref()
                .unwrap_or("<missing>")
                .to_string(),
        }
    })
}

fn information_requirement_href(
    decision: &DmnDecisionDefinition,
    requirement: &DmnInformationRequirementReference,
) -> Result<String> {
    let href = requirement.href.as_deref().unwrap_or("<missing>");
    href.strip_prefix('#')
        .filter(|target| !target.is_empty())
        .map(ToString::to_string)
        .ok_or_else(
            || BpmnEngineError::UnsupportedDmnInformationRequirementHref {
                source_id: decision.source_id.to_string(),
                decision_id: decision.decision.decision_id.to_string(),
                href: href.to_string(),
            },
        )
}

fn merge_evaluation_output(variables: &mut Value, output: &Value) {
    let (Some(variables), Some(output)) = (variables.as_object_mut(), output.as_object()) else {
        return;
    };
    for (key, value) in output {
        variables.insert(key.clone(), value.clone());
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
