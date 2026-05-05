//! DMN invocation evaluation leaf.

use super::support::{is_simple_identifier, knowledge_requirement_href};
use crate::dmn::evaluate_dmn_literal_expression;
use crate::dmn_model_api::{
    DmnBusinessKnowledgeModelDefinition, DmnDecisionDefinition, DmnEvaluationResult, DmnInvocation,
    DmnKnowledgeRequirementReference,
};
use crate::error::{BpmnEngineError, Result};
use crate::ir::BpmnPackage;
use serde_json::{Map, Value};

pub(super) fn evaluate_dmn_invocation(
    package: &BpmnPackage,
    decision: &DmnDecisionDefinition,
    invocation: &DmnInvocation,
    variables: &Value,
) -> Result<DmnEvaluationResult> {
    let target_name = invocation_target_name(invocation)?;
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

fn invocation_target_name(invocation: &DmnInvocation) -> Result<String> {
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
) -> Result<&'a DmnBusinessKnowledgeModelDefinition> {
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

fn resolve_required_knowledge_targets<'a>(
    package: &'a BpmnPackage,
    decision: &DmnDecisionDefinition,
) -> Result<Option<Vec<&'a DmnBusinessKnowledgeModelDefinition>>> {
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

fn resolve_required_knowledge_definition<'a>(
    package: &'a BpmnPackage,
    decision: &DmnDecisionDefinition,
    requirement: &DmnKnowledgeRequirementReference,
) -> Result<&'a DmnBusinessKnowledgeModelDefinition> {
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
