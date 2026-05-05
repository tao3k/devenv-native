use super::core::evaluate_dmn_package_decision_sync;
use super::support::merge_evaluation_output;
use crate::dmn_model_api::{
    DmnDecisionDefinition, DmnDecisionRef, DmnDecisionServiceDefinition,
    DmnDecisionServiceReference, DmnEvaluationRequest, DmnEvaluationResult, DmnInputDataDefinition,
};
use crate::error::{BpmnEngineError, Result};
use crate::ir::BpmnPackage;
use serde_json::{Map, Value};

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

pub(super) fn resolve_decision_service_output_decisions<'a>(
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

pub(super) fn evaluate_dmn_package_decision_service_outputs_sync(
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

pub(super) fn validate_decision_service_exposure_contract(
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
) -> Result<&'a DmnInputDataDefinition> {
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
