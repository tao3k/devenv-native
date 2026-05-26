use super::resume::prepare_resume_workflow;
use crate::bpmn::control::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowTaskCompleteBatchReport, QianjiBpmnWorkflowTaskCompleteBatchRequest,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
};
use crate::bpmn::driver::QianjiBpmnPendingHostCompletion;
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::execution::QianjiBpmnExecutionFacade;
use xiuxian_qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnHostBridge, ManualTaskOutcome, PendingHostWorkResult,
    ScriptTaskOutcome, SendTaskOutcome, ServiceTaskOutcome, UserTaskOutcome,
};

pub(crate) async fn complete_workflow_task<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
    let resume_request = QianjiBpmnWorkflowResumeRequest {
        bpmn_path: request.bpmn_path.clone(),
        dmn_paths: request.dmn_paths.clone(),
        instance_id: request.instance_id.clone(),
        checkpoint_backend: request.checkpoint_backend.clone(),
    };
    let prepared = prepare_resume_workflow(service, &resume_request).await?;
    complete_prepared_workflow_task(service, prepared, request, host).await
}

pub(crate) async fn complete_prepared_workflow_task<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let loaded_checkpoint = prepared.loaded_checkpoint.clone();
    validate_completion_claimant(loaded_checkpoint.as_ref(), &request.completion)?;
    let completion = pending_host_completion_from_completion(&request.completion);
    let execution = match (request.continue_until_human_boundary, loaded_checkpoint) {
        (true, Some(checkpoint)) => {
            execution_facade
                .complete_pending_host_work_from_checkpoint_until_human_boundary(
                    &prepared.execution_request,
                    checkpoint,
                    completion,
                    host,
                )
                .await?
        }
        (false, Some(checkpoint)) => {
            execution_facade
                .complete_pending_host_work_from_checkpoint(
                    &prepared.execution_request,
                    checkpoint,
                    completion,
                    host,
                )
                .await?
        }
        (true, None) => {
            execution_facade
                .complete_pending_host_work_until_human_boundary(
                    &prepared.execution_request,
                    completion,
                    host,
                )
                .await?
        }
        (false, None) => {
            execution_facade
                .complete_pending_host_work(&prepared.execution_request, completion, host)
                .await?
        }
    };

    Ok(QianjiBpmnWorkflowTaskCompleteReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn complete_prepared_workflow_task_until_host_boundary<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowControlError> {
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let loaded_checkpoint = prepared.loaded_checkpoint.clone();
    validate_completion_claimant(loaded_checkpoint.as_ref(), &request.completion)?;
    let completion = pending_host_completion_from_completion(&request.completion);
    let execution = match loaded_checkpoint {
        Some(checkpoint) => {
            execution_facade
                .complete_pending_host_work_from_checkpoint_until_host_boundary(
                    &prepared.execution_request,
                    checkpoint,
                    completion,
                    host,
                )
                .await?
        }
        None => {
            execution_facade
                .complete_pending_host_work_until_host_boundary(
                    &prepared.execution_request,
                    completion,
                    host,
                )
                .await?
        }
    };

    Ok(QianjiBpmnWorkflowTaskCompleteReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

pub(crate) async fn complete_prepared_workflow_task_batch_until_host_boundary<H: BpmnHostBridge>(
    service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    request: &QianjiBpmnWorkflowTaskCompleteBatchRequest,
    host: &H,
) -> Result<QianjiBpmnWorkflowTaskCompleteBatchReport, QianjiBpmnWorkflowControlError> {
    let mut execution_facade =
        QianjiBpmnExecutionFacade::new(prepared.package, prepared.checkpoint_store.clone());
    if let Some(scheduler_identity) = service.scheduler_identity.clone() {
        execution_facade = execution_facade.with_scheduler_identity(scheduler_identity);
    }
    let loaded_checkpoint = prepared.loaded_checkpoint.clone().ok_or_else(|| {
        QianjiBpmnWorkflowControlError::CheckpointMissing {
            instance_id: request.instance_id.to_string(),
        }
    })?;
    for completion in &request.completions {
        validate_completion_claimant(Some(&loaded_checkpoint), completion)?;
    }
    let completions = request
        .completions
        .iter()
        .map(pending_host_completion_from_completion)
        .collect::<Vec<_>>();
    let execution = execution_facade
        .complete_pending_host_work_batch_from_checkpoint_until_host_boundary(
            &prepared.execution_request,
            loaded_checkpoint,
            completions,
            host,
        )
        .await?;

    Ok(QianjiBpmnWorkflowTaskCompleteBatchReport {
        resolved_bpmn_path: prepared.resolved_bpmn_path,
        resolved_dmn_paths: prepared.resolved_dmn_paths,
        checkpoint_store: prepared.checkpoint_store,
        execution,
    })
}

fn validate_completion_claimant(
    checkpoint: Option<&BpmnCheckpointEnvelope>,
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> Result<(), QianjiBpmnWorkflowControlError> {
    let Some(checkpoint) = checkpoint else {
        return Ok(());
    };
    let Some(pending) = checkpoint
        .state
        .pending_host_work
        .iter()
        .find(|work| work.token_id == completion.token_id)
    else {
        return Ok(());
    };
    let Some(claim) = pending.claim.as_ref() else {
        return Ok(());
    };

    let Some(actual_claimant) = completion
        .claimant
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Err(BpmnOrchestrationError::PendingHostWorkClaimRequired {
            instance_id: checkpoint.state.instance_id.as_ref().into(),
            token_id: completion.token_id,
            claimed_by: claim.claimant.clone(),
        }
        .into());
    };

    if actual_claimant == claim.claimant {
        return Ok(());
    }

    Err(BpmnOrchestrationError::PendingHostWorkClaimantMismatch {
        instance_id: checkpoint.state.instance_id.as_ref().into(),
        token_id: completion.token_id,
        expected_claimant: claim.claimant.clone(),
        actual_claimant: actual_claimant.to_string(),
    }
    .into())
}

fn pending_host_completion_from_completion(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> QianjiBpmnPendingHostCompletion {
    QianjiBpmnPendingHostCompletion::new(
        completion.token_id,
        completion.process_id.clone(),
        completion.activity_id.clone(),
        pending_host_work_result_from_completion(completion),
    )
}

fn pending_host_work_result_from_completion(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> PendingHostWorkResult {
    match completion.kind {
        QianjiBpmnWorkflowTaskCompletionKind::Send => {
            PendingHostWorkResult::Send(SendTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::Service => {
            PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::Script => {
            PendingHostWorkResult::Script(ScriptTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::User => {
            PendingHostWorkResult::User(UserTaskOutcome {
                data: completion.data.clone(),
            })
        }
        QianjiBpmnWorkflowTaskCompletionKind::Manual => {
            PendingHostWorkResult::Manual(ManualTaskOutcome {
                data: completion.data.clone(),
            })
        }
    }
}
