//! Bounded DMN evaluation core.

use super::support::merge_evaluation_output;
use super::{invocation, rule};
use crate::dmn::{
    evaluate_dmn_context_expression_decision, evaluate_dmn_list_expression_decision,
    evaluate_dmn_literal_expression_decision, evaluate_dmn_relation_expression_decision,
};
use crate::ir::BpmnPackage;
use crate::{
    BpmnEngineError, DmnDecisionDefinition, DmnDecisionRef, DmnEvaluationRequest,
    DmnEvaluationResult, DmnHitPolicy, DmnInformationRequirementReference,
};
use serde_json::{Map, Value};
use std::sync::Arc;

type Result<T> = std::result::Result<T, BpmnEngineError>;

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
    if decision.invocation.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "evaluate_dmn_invocation_without_package_context",
        });
    }

    let mut matched_rule_ids = Vec::new();
    match decision.table.hit_policy {
        DmnHitPolicy::Unique => {
            for rule in &decision.table.rules {
                if rule::rule_matches(decision, rule, &request.variables) {
                    matched_rule_ids.push(Arc::clone(&rule.rule_id));
                    return Ok(DmnEvaluationResult::new(
                        decision.decision.decision_id.as_ref(),
                        rule::unique_rule_output(decision, rule),
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
                if !rule::rule_matches(decision, rule, &request.variables) {
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
            return invocation::evaluate_dmn_invocation(package, decision, invocation, &variables);
        }
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
