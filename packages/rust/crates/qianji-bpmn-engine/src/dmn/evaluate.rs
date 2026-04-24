//! Bounded DMN evaluation internals.

use super::literal_expression::evaluate_dmn_literal_expression;
use crate::dmn::{
    evaluate_dmn_context_expression_decision, evaluate_dmn_list_expression_decision,
    evaluate_dmn_literal_expression_decision, evaluate_dmn_relation_expression_decision,
};
use crate::dmn_duration::{
    DmnDurationValue, parse_day_time_duration_str, parse_year_month_duration_str,
};
use crate::dmn_model_api::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateTimeComparison,
    DmnDateTimeRange, DmnDecisionDefinition, DmnDecisionRef, DmnDecisionServiceDefinition,
    DmnDecisionServiceReference, DmnDurationComparison, DmnDurationRange, DmnEvaluationRequest,
    DmnEvaluationResult, DmnHitPolicy, DmnInformationRequirementReference, DmnInputClause,
    DmnInputEntry, DmnKnowledgeRequirementReference, DmnNumericRange, DmnRule, DmnTimeComparison,
    DmnTimeRange,
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

/// Synchronous bounded package-aware DMN binding entrypoint for in-engine
/// runtime paths that can consume either one registered local decision or one
/// bounded local decision-service alias.
pub(crate) fn evaluate_dmn_package_binding_sync(
    package: &BpmnPackage,
    decision_ref: &DmnDecisionRef,
    variables: &Value,
) -> Result<Option<DmnEvaluationResult>> {
    if let Some(definition) = package.find_dmn_decision(decision_ref)? {
        return evaluate_dmn_package_decision_sync(
            package,
            definition,
            &DmnEvaluationRequest::new(decision_ref.clone(), variables.clone()),
        )
        .map(Some);
    }
    let Some(decision_service) = package.find_dmn_decision_service(decision_ref)? else {
        return Ok(None);
    };
    let output_decisions = resolve_decision_service_output_decisions(package, decision_service)?;
    validate_decision_service_exposure_contract(package, decision_service)?;
    if let [output_decision] = output_decisions.as_slice() {
        return evaluate_dmn_package_decision_sync(
            package,
            output_decision,
            &DmnEvaluationRequest::new(output_decision.decision.clone(), variables.clone()),
        )
        .map(Some);
    }

    evaluate_dmn_package_decision_service_outputs_sync(
        package,
        decision_service,
        &output_decisions,
        variables,
    )
    .map(Some)
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
    if decision.invocation.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "evaluate_dmn_invocation_without_package_context",
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
        if let Some(invocation) = decision.invocation.as_ref() {
            return evaluate_dmn_invocation(package, decision, invocation, &variables);
        }
        evaluate_dmn_decision_sync(
            decision,
            &DmnEvaluationRequest::new(request.decision.clone(), variables),
        )
    })();
    let _ = stack.pop();
    result
}

fn evaluate_dmn_invocation(
    package: &BpmnPackage,
    decision: &DmnDecisionDefinition,
    invocation: &crate::dmn_model_api::DmnInvocation,
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    let target_name = invocation_target_name(decision, invocation)?;
    let target =
        resolve_invocation_business_knowledge_model(package, decision, target_name.as_str())?;
    let logic =
        target
            .encapsulated_logic
            .as_ref()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "evaluate_dmn_invocation_without_encapsulated_logic",
            })?;
    let body = logic
        .body
        .as_ref()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "evaluate_dmn_invocation_without_bkm_literal_body",
        })?;
    let body_text = body
        .text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "evaluate_dmn_invocation_without_bkm_literal_body",
        })?;

    let mut scope = variables.as_object().cloned().unwrap_or_default();
    for binding in &invocation.bindings {
        let parameter_name = binding
            .parameter
            .as_ref()
            .and_then(|parameter| parameter.name.as_deref())
            .filter(|name| is_simple_identifier(name))
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "evaluate_dmn_invocation_without_named_parameter",
            })?;
        let argument = binding
            .argument
            .as_ref()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "evaluate_dmn_invocation_without_argument",
            })?;
        let value = evaluate_dmn_literal_expression(
            decision.source_id.as_ref(),
            argument.text.as_ref(),
            variables,
        )?;
        scope.insert(parameter_name.to_string(), value);
    }

    let value = evaluate_dmn_literal_expression(
        decision.source_id.as_ref(),
        body_text,
        &Value::Object(scope),
    )?;
    let mut output = Map::new();
    output.insert(decision.decision.decision_id.to_string(), value);
    Ok(DmnEvaluationResult::new(
        decision.decision.decision_id.as_ref(),
        Value::Object(output),
        Vec::new(),
    ))
}

fn invocation_target_name(
    _decision: &DmnDecisionDefinition,
    invocation: &crate::dmn_model_api::DmnInvocation,
) -> Result<String> {
    invocation
        .invoked_expression
        .as_ref()
        .map(|expression| expression.text.trim())
        .filter(|text| is_simple_identifier(text))
        .map(ToString::to_string)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "evaluate_dmn_invocation_unsupported_target_expression",
        })
}

fn resolve_invocation_business_knowledge_model<'a>(
    package: &'a BpmnPackage,
    decision: &DmnDecisionDefinition,
    target_name: &str,
) -> Result<&'a crate::dmn_model_api::DmnBusinessKnowledgeModelDefinition> {
    let allowed_targets = resolve_required_knowledge_targets(package, decision)?;
    let matches = match allowed_targets.as_ref() {
        Some(allowed_targets) => allowed_targets
            .iter()
            .copied()
            .filter(|business_knowledge_model| {
                business_knowledge_model
                    .business_knowledge_model_id
                    .as_deref()
                    == Some(target_name)
                    || business_knowledge_model.variable_name.as_deref() == Some(target_name)
            })
            .collect::<Vec<_>>(),
        None => package
            .dmn_business_knowledge_models()
            .iter()
            .filter(|business_knowledge_model| {
                business_knowledge_model.source_id.as_ref() == decision.source_id.as_ref()
                    && (business_knowledge_model
                        .business_knowledge_model_id
                        .as_deref()
                        == Some(target_name)
                        || business_knowledge_model.variable_name.as_deref() == Some(target_name))
            })
            .collect::<Vec<_>>(),
    };

    match matches.as_slice() {
        [target] => Ok(*target),
        [] => Err(if allowed_targets.is_some() {
            BpmnEngineError::UndeclaredDmnInvocationKnowledgeTarget {
                source_id: decision.source_id.to_string(),
                decision_id: decision.decision.decision_id.to_string(),
                target: target_name.to_string(),
            }
        } else {
            BpmnEngineError::MissingDmnInvocationTarget {
                source_id: decision.source_id.to_string(),
                decision_id: decision.decision.decision_id.to_string(),
                target: target_name.to_string(),
            }
        }),
        _ => Err(BpmnEngineError::AmbiguousDmnInvocationTarget {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
            target: target_name.to_string(),
            count: matches.len(),
        }),
    }
}

fn resolve_decision_service_output_decisions<'a>(
    package: &'a BpmnPackage,
    decision_service: &DmnDecisionServiceDefinition,
) -> Result<Vec<&'a DmnDecisionDefinition>> {
    if decision_service.output_decisions.is_empty() {
        return Err(BpmnEngineError::UnsupportedDmnDecisionServiceOutputCount {
            source_id: decision_service.source_id.to_string(),
            decision_service_id: decision_service_id_for_error(decision_service),
            count: decision_service.output_decisions.len(),
        });
    }

    decision_service
        .output_decisions
        .iter()
        .map(|output_reference| {
            let target_id = decision_service_output_href(decision_service, output_reference)?;
            let decision_ref =
                DmnDecisionRef::new(target_id).with_source_id(decision_service.source_id.as_ref());
            package.find_dmn_decision(&decision_ref)?.ok_or_else(|| {
                BpmnEngineError::MissingDmnDecisionServiceOutputTarget {
                    source_id: decision_service.source_id.to_string(),
                    decision_service_id: decision_service_id_for_error(decision_service),
                    href: output_reference
                        .href
                        .as_deref()
                        .unwrap_or("<missing>")
                        .to_string(),
                }
            })
        })
        .collect()
}

fn evaluate_dmn_package_decision_service_outputs_sync(
    package: &BpmnPackage,
    decision_service: &DmnDecisionServiceDefinition,
    output_decisions: &[&DmnDecisionDefinition],
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    let mut output = Value::Object(Map::new());
    let mut matched_rule_ids = Vec::new();

    for output_decision in output_decisions {
        let evaluation = evaluate_dmn_package_decision_sync(
            package,
            output_decision,
            &DmnEvaluationRequest::new(output_decision.decision.clone(), variables.clone()),
        )?;
        merge_evaluation_output(&mut output, &evaluation.output);
        matched_rule_ids.extend(evaluation.matched_rule_ids);
    }

    Ok(DmnEvaluationResult::new(
        decision_service_id_for_error(decision_service),
        output,
        matched_rule_ids,
    ))
}

fn resolve_required_knowledge_targets<'a>(
    package: &'a BpmnPackage,
    decision: &DmnDecisionDefinition,
) -> Result<Option<Vec<&'a crate::dmn_model_api::DmnBusinessKnowledgeModelDefinition>>> {
    if decision.knowledge_requirements.is_empty() {
        return Ok(None);
    }

    let mut targets = Vec::with_capacity(decision.knowledge_requirements.len());
    for requirement in &decision.knowledge_requirements {
        targets.push(resolve_required_knowledge_definition(
            package,
            decision,
            requirement,
        )?);
    }
    Ok(Some(targets))
}

fn validate_decision_service_exposure_contract(
    package: &BpmnPackage,
    decision_service: &DmnDecisionServiceDefinition,
) -> Result<()> {
    for reference in &decision_service.encapsulated_decisions {
        let _ = resolve_decision_service_decision_reference(package, decision_service, reference)?;
    }
    for reference in &decision_service.input_decisions {
        let _ = resolve_decision_service_decision_reference(package, decision_service, reference)?;
    }
    for reference in &decision_service.input_data {
        let _ =
            resolve_decision_service_input_data_reference(package, decision_service, reference)?;
    }
    Ok(())
}

fn decision_service_output_href(
    decision_service: &DmnDecisionServiceDefinition,
    reference: &DmnDecisionServiceReference,
) -> Result<String> {
    let href = reference.href.as_deref().unwrap_or("<missing>");
    href.strip_prefix('#')
        .filter(|target| !target.is_empty())
        .map(ToString::to_string)
        .ok_or_else(
            || BpmnEngineError::UnsupportedDmnDecisionServiceOutputHref {
                source_id: decision_service.source_id.to_string(),
                decision_service_id: decision_service_id_for_error(decision_service),
                href: href.to_string(),
            },
        )
}

fn resolve_decision_service_decision_reference<'a>(
    package: &'a BpmnPackage,
    decision_service: &DmnDecisionServiceDefinition,
    reference: &DmnDecisionServiceReference,
) -> Result<&'a DmnDecisionDefinition> {
    let target_id = decision_service_reference_target_id(decision_service, reference)?;
    let decision_ref =
        DmnDecisionRef::new(&target_id).with_source_id(decision_service.source_id.as_ref());
    package.find_dmn_decision(&decision_ref)?.ok_or_else(|| {
        BpmnEngineError::MissingDmnDecisionServiceReferenceTarget {
            source_id: decision_service.source_id.to_string(),
            decision_service_id: decision_service_id_for_error(decision_service),
            reference_kind: reference.reference_kind.to_string(),
            href: reference.href.as_deref().unwrap_or("<missing>").to_string(),
        }
    })
}

fn resolve_decision_service_input_data_reference<'a>(
    package: &'a BpmnPackage,
    decision_service: &DmnDecisionServiceDefinition,
    reference: &DmnDecisionServiceReference,
) -> Result<&'a crate::dmn_model_api::DmnInputDataDefinition> {
    let target_id = decision_service_reference_target_id(decision_service, reference)?;
    package
        .find_dmn_input_data(decision_service.source_id.as_ref(), &target_id)
        .ok_or_else(
            || BpmnEngineError::MissingDmnDecisionServiceReferenceTarget {
                source_id: decision_service.source_id.to_string(),
                decision_service_id: decision_service_id_for_error(decision_service),
                reference_kind: reference.reference_kind.to_string(),
                href: reference.href.as_deref().unwrap_or("<missing>").to_string(),
            },
        )
}

fn decision_service_reference_target_id(
    decision_service: &DmnDecisionServiceDefinition,
    reference: &DmnDecisionServiceReference,
) -> Result<String> {
    let href = reference.href.as_deref().unwrap_or("<missing>");
    href.strip_prefix('#')
        .filter(|target| !target.is_empty())
        .map(ToString::to_string)
        .ok_or_else(
            || BpmnEngineError::UnsupportedDmnDecisionServiceReferenceHref {
                source_id: decision_service.source_id.to_string(),
                decision_service_id: decision_service_id_for_error(decision_service),
                reference_kind: reference.reference_kind.to_string(),
                href: href.to_string(),
            },
        )
}

fn decision_service_id_for_error(decision_service: &DmnDecisionServiceDefinition) -> String {
    decision_service
        .decision_service_id
        .as_deref()
        .unwrap_or("<missing>")
        .to_string()
}

fn resolve_required_knowledge_definition<'a>(
    package: &'a BpmnPackage,
    decision: &DmnDecisionDefinition,
    requirement: &DmnKnowledgeRequirementReference,
) -> Result<&'a crate::dmn_model_api::DmnBusinessKnowledgeModelDefinition> {
    let target_id = knowledge_requirement_href(decision, requirement)?;
    package
        .find_dmn_business_knowledge_model(decision.source_id.as_ref(), &target_id)
        .ok_or_else(|| BpmnEngineError::MissingDmnRequiredKnowledgeTarget {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
            href: requirement
                .href
                .as_deref()
                .unwrap_or("<missing>")
                .to_string(),
        })
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

fn knowledge_requirement_href(
    decision: &DmnDecisionDefinition,
    requirement: &DmnKnowledgeRequirementReference,
) -> Result<String> {
    let href = requirement.href.as_deref().unwrap_or("<missing>");
    href.strip_prefix('#')
        .filter(|target| !target.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| BpmnEngineError::UnsupportedDmnKnowledgeRequirementHref {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
            href: href.to_string(),
        })
}

fn merge_evaluation_output(variables: &mut Value, output: &Value) {
    let (Some(variables), Some(output)) = (variables.as_object_mut(), output.as_object()) else {
        return;
    };
    for (key, value) in output {
        variables.insert(key.clone(), value.clone());
    }
}

fn is_simple_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
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
