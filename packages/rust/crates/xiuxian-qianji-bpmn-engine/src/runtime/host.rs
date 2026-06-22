//! Runtime helpers for host-boundary request materialization.

use super::{
    BpmnInstanceState, PendingHostWork, PendingHostWorkClaim,
    parallel_multi_instance_iteration_variables, sequential_multi_instance_iteration_variables,
};
use crate::dmn_model_api::DmnEvaluationRequest;
use crate::error::{BpmnEngineError, Result};
use crate::host_types_api::{
    BpmnHostActivityId, BpmnHostProcessId, BusinessRuleTaskRequest, ManualTaskRequest,
    ParallelMultiInstanceContext, PendingHostWorkRequest, RepeatExecutionContext,
    ScriptTaskRequest, SendTaskRequest, SequentialMultiInstanceContext, ServiceTaskRequest,
    TaskRequest, UserTaskRequest,
};
use crate::ir_node_api::{
    BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec, BpmnLaneMembershipSpec,
    BpmnTaskInputSource, BpmnTaskIoSpec, BpmnTaskOutputBinding,
};
use serde_json::{Map, Value};

/// Builds a typed host-dispatch request from the currently blocked BPMN
/// instance state.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work, or
/// [`BpmnEngineError::AmbiguousPendingHostWork`] when more than one pending
/// host-work entry exists.
pub(crate) fn build_pending_host_work_request_impl(
    instance: &BpmnInstanceState,
) -> Result<PendingHostWorkRequest> {
    let requests = build_pending_host_work_requests_impl(instance)?;
    match requests.as_slice() {
        [] => Err(BpmnEngineError::MissingPendingHostWork {
            instance_id: (instance.instance_id.to_string()).into(),
        }),
        [request] => Ok(request.clone()),
        requests => Err(BpmnEngineError::AmbiguousPendingHostWork {
            instance_id: (instance.instance_id.to_string()).into(),
            count: requests.len(),
        }),
    }
}

/// Builds typed host-dispatch requests from every currently blocked BPMN token.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work.
pub(crate) fn build_pending_host_work_requests_impl(
    instance: &BpmnInstanceState,
) -> Result<Vec<PendingHostWorkRequest>> {
    if instance.pending_host_work.is_empty() {
        return Err(BpmnEngineError::MissingPendingHostWork {
            instance_id: (instance.instance_id.to_string()).into(),
        });
    }

    instance
        .pending_host_work
        .iter()
        .map(|pending| build_pending_host_work_request_for_entry(instance, pending))
        .collect()
}

fn build_pending_host_work_request_for_entry(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
) -> Result<PendingHostWorkRequest> {
    let envelope = build_host_task_request_envelope(instance, pending)?;

    Ok(match pending.kind {
        super::PendingHostWorkKind::Task => PendingHostWorkRequest::Task(TaskRequest {
            instance_id: envelope.instance_id.into(),
            process_id: envelope.process_id,
            token_id: envelope.token_id.into(),
            node_index: envelope.node_index,
            activity_id: envelope.activity_id,
            variables: envelope.variables,
            inputs: envelope.inputs,
            output_bindings: envelope.output_bindings,
            repeat: envelope.repeat,
            lane: envelope.lane,
        }),
        super::PendingHostWorkKind::Send => PendingHostWorkRequest::Send(SendTaskRequest {
            instance_id: envelope.instance_id,
            token_id: envelope.token_id,
            node_index: envelope.node_index,
            message_reference: pending.event_reference.clone().ok_or(
                BpmnEngineError::UnsupportedOperation {
                    operation: "build_pending_send_task_request_missing_message_reference",
                },
            )?,
            message_name: pending.event_name.clone(),
            variables: envelope.variables,
            inputs: envelope.inputs,
            output_bindings: envelope.output_bindings,
        }),
        super::PendingHostWorkKind::Service => {
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: envelope.instance_id,
                token_id: envelope.token_id,
                node_index: envelope.node_index,
                variables: envelope.variables,
                inputs: envelope.inputs,
                output_bindings: envelope.output_bindings,
                repeat: envelope.repeat,
            })
        }
        super::PendingHostWorkKind::Script => PendingHostWorkRequest::Script(ScriptTaskRequest {
            instance_id: envelope.instance_id,
            token_id: envelope.token_id,
            node_index: envelope.node_index,
            script_format: pending.script_format.clone(),
            script_body: pending.script_body.clone(),
            variables: envelope.variables,
            inputs: envelope.inputs,
            output_bindings: envelope.output_bindings,
            repeat: envelope.repeat,
        }),
        super::PendingHostWorkKind::User => PendingHostWorkRequest::User(UserTaskRequest {
            instance_id: envelope.instance_id.into(),
            process_id: envelope.process_id,
            token_id: envelope.token_id.into(),
            node_index: envelope.node_index,
            activity_id: envelope.activity_id,
            variables: envelope.variables,
            inputs: envelope.inputs,
            output_bindings: envelope.output_bindings,
            repeat: envelope.repeat,
            lane: envelope.lane,
            form: envelope.form,
            assignment: envelope.assignment,
            claim: envelope.claim,
        }),
        super::PendingHostWorkKind::Manual => PendingHostWorkRequest::Manual(ManualTaskRequest {
            instance_id: envelope.instance_id.into(),
            process_id: envelope.process_id,
            token_id: envelope.token_id.into(),
            node_index: envelope.node_index,
            activity_id: envelope.activity_id,
            variables: envelope.variables,
            inputs: envelope.inputs,
            output_bindings: envelope.output_bindings,
            repeat: envelope.repeat,
            lane: envelope.lane,
            form: envelope.form,
            assignment: envelope.assignment,
            claim: envelope.claim,
        }),
        super::PendingHostWorkKind::BusinessRule => {
            build_business_rule_task_request(instance, pending, envelope)?
        }
    })
}

fn build_host_task_request_envelope(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
) -> Result<HostTaskRequestEnvelope> {
    let node_index = pending.node_index;
    let process_id = pending
        .process_id
        .clone()
        .unwrap_or_else(|| instance.process.process_id.as_ref().to_string().into());
    let activity_id = pending
        .activity_id
        .clone()
        .unwrap_or_else(|| format!("node#{node_index}").into());
    let (variables, repeat) = resolve_pending_host_work_execution_context(instance, pending)?;
    let inputs = resolve_task_inputs(pending, &variables)?;
    Ok(HostTaskRequestEnvelope {
        instance_id: instance.instance_id.to_string(),
        process_id,
        token_id: pending.token_id,
        node_index,
        activity_id,
        variables,
        inputs,
        output_bindings: task_output_bindings(pending),
        repeat,
        lane: pending.lane.clone(),
        form: pending.human_task_form.clone(),
        assignment: pending.human_task_assignment.clone(),
        claim: pending.claim.clone(),
    })
}

fn resolve_pending_host_work_execution_context(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
) -> Result<(serde_json::Value, Option<RepeatExecutionContext>)> {
    let uses_active_process_context = pending
        .process_id
        .as_deref()
        .is_none_or(|process_id| process_id == instance.process.process_id.as_ref());
    if !uses_active_process_context {
        return Ok((instance.variables.clone(), None));
    }

    sequential_multi_instance_iteration_variables(
        instance,
        pending.node_index,
        &instance.variables,
    )?
    .map_or_else(
        || {
            parallel_multi_instance_iteration_variables(
                instance,
                pending.node_index,
                pending.token_id,
                &instance.variables,
            )
            .map(|repeat| {
                repeat.map(|(iteration_index, total_iterations, variables)| {
                    (
                        variables,
                        Some(RepeatExecutionContext::ParallelMultiInstance(
                            ParallelMultiInstanceContext {
                                iteration_index,
                                total_iterations,
                            },
                        )),
                    )
                })
            })
        },
        |(iteration_index, total_iterations, variables)| {
            Ok(Some((
                variables,
                Some(RepeatExecutionContext::SequentialMultiInstance(
                    SequentialMultiInstanceContext {
                        iteration_index,
                        total_iterations,
                    },
                )),
            )))
        },
    )
    .map(|context| context.unwrap_or_else(|| (instance.variables.clone(), None)))
}

struct HostTaskRequestEnvelope {
    instance_id: String,
    process_id: BpmnHostProcessId,
    token_id: u64,
    node_index: u32,
    activity_id: BpmnHostActivityId,
    variables: Value,
    inputs: Value,
    output_bindings: Vec<BpmnTaskOutputBinding>,
    repeat: Option<RepeatExecutionContext>,
    lane: Option<BpmnLaneMembershipSpec>,
    form: Option<BpmnHumanTaskFormSpec>,
    assignment: Option<BpmnHumanTaskAssignmentSpec>,
    claim: Option<PendingHostWorkClaim>,
}

fn build_business_rule_task_request(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
    envelope: HostTaskRequestEnvelope,
) -> Result<PendingHostWorkRequest> {
    let decision = pending.decision.clone().ok_or_else(|| {
        BpmnEngineError::MissingBusinessRuleDecisionRef {
            process_id: pending
                .process_id
                .clone()
                .map_or_else(
                    || instance.process.process_id.to_string(),
                    |id| id.to_string(),
                )
                .into(),
            node_id: (pending.node_index.to_string()).into(),
        }
    })?;
    let evaluation_variables = if pending
        .task_io
        .as_ref()
        .is_some_and(|task_io| !task_io.inputs.is_empty())
    {
        envelope.inputs.clone()
    } else {
        envelope.variables
    };
    Ok(PendingHostWorkRequest::BusinessRule(
        BusinessRuleTaskRequest {
            instance_id: (envelope.instance_id),
            token_id: (envelope.token_id),
            node_index: envelope.node_index,
            evaluation: DmnEvaluationRequest::new(decision, evaluation_variables),
            inputs: envelope.inputs,
            output_bindings: envelope.output_bindings,
            repeat: envelope.repeat,
        },
    ))
}

fn task_output_bindings(pending: &PendingHostWork) -> Vec<BpmnTaskOutputBinding> {
    pending
        .task_io
        .as_ref()
        .map_or_else(Vec::new, |task_io| task_io.outputs.clone())
}

fn resolve_task_inputs(pending: &PendingHostWork, variables: &Value) -> Result<Value> {
    let Some(task_io) = pending.task_io.as_ref() else {
        return Ok(Value::Object(Map::new()));
    };
    materialize_task_inputs(task_io, variables, pending)
}

fn materialize_task_inputs(
    task_io: &BpmnTaskIoSpec,
    variables: &Value,
    pending: &PendingHostWork,
) -> Result<Value> {
    let mut inputs = Map::new();
    for input in &task_io.inputs {
        let value = match &input.source {
            BpmnTaskInputSource::Variable { source_ref } => {
                resolve_value_path(variables, source_ref.as_ref())
                    .cloned()
                    .ok_or_else(|| BpmnEngineError::UnresolvedTaskInputSource {
                        process_id: pending
                            .process_id
                            .clone()
                            .map_or_else(|| "<active>".to_string(), |id| id.to_string())
                            .into(),
                        node_index: pending.node_index,
                        input: input.name.to_string(),
                        source_ref: source_ref.to_string(),
                    })?
            }
            BpmnTaskInputSource::Literal { value } => parse_literal_input_value(value.as_ref()),
        };
        inputs.insert(input.name.to_string(), value);
    }
    Ok(Value::Object(inputs))
}

fn parse_literal_input_value(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()))
}

fn resolve_value_path<'a>(variables: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = variables;
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        current = current.get(segment)?;
    }
    Some(current)
}
