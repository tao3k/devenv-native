use super::activity_evidence::{
    QianjiBpmnWorkflowCompletionActivityEvidenceInput,
    QianjiBpmnWorkflowFailureActivityEvidenceInput, matching_pending_work_for_completion,
    matching_pending_work_for_failure, now_unix_ms, record_completion_activity_evidence,
    record_failure_activity_evidence,
};
use super::error_api::QianjiBpmnWorkflowHttpError;
use super::request_api::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowStartHttpRequest,
    QianjiBpmnWorkflowStatusHttpQuery, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskFailHttpRequest, QianjiBpmnWorkflowTaskFailureHttpPayload,
    QianjiBpmnWorkflowTaskReleaseHttpRequest, QianjiControlRecoveryApplyHttpRequest,
};
use super::response_api::{
    QianjiBpmnWorkflowCancelHttpResponse, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiControlDiagnosticsHttpResponse,
    QianjiControlHistoryHttpResponse, QianjiControlRecoveryApplyHttpResponse,
    QianjiControlRecoveryHttpResponse, QianjiControlRunSummaryHttpResponse,
};
use super::state::QianjiBpmnWorkflowHttpState;
use crate::bpmn::control::{
    QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowTaskCompleteReport,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionPayload,
};
use crate::bpmn::identity::QianjiBpmnWorkflowInstanceId;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use xiuxian_qianji_bpmn_engine::BpmnHostBridge;
use xiuxian_qianji_control::{
    ControlLedger, HotStateStore, RecoveryAttempt, RecoveryLoopApplicationRequest, RecoveryPolicy,
    RunId, apply_recovery_plan,
};

pub(super) fn router<H>(state: QianjiBpmnWorkflowHttpState<H>) -> Router
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/workflows/start", post(start_workflow::<H>))
        .route("/workflows/{instance_id}", get(load_workflow_status::<H>))
        .route(
            "/workflows/{instance_id}/resume",
            post(resume_workflow::<H>),
        )
        .route(
            "/workflows/{instance_id}/cancel",
            post(cancel_workflow::<H>),
        )
        .route(
            "/workflows/{instance_id}/events/poll",
            post(poll_workflow_events::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/complete",
            post(complete_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/complete-batch",
            post(complete_workflow_tasks_batch::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/fail",
            post(fail_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/claim",
            post(claim_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/release",
            post(release_workflow_task::<H>),
        )
        .route(
            "/control/runs/{run_id}/history",
            get(load_control_history::<H>),
        )
        .route(
            "/control/runs/{run_id}/summary",
            get(load_control_summary::<H>),
        )
        .route(
            "/control/runs/{run_id}/recovery",
            get(load_control_recovery::<H>),
        )
        .route(
            "/control/runs/{run_id}/diagnostics",
            get(load_control_diagnostics::<H>),
        )
        .route(
            "/control/runs/{run_id}/recovery/apply",
            post(apply_control_recovery::<H>),
        )
        .with_state(state)
}

async fn start_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Json(request): Json<QianjiBpmnWorkflowStartHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_control_request();
    let prepared = state.service.prepare_start_workflow(&request)?;
    let report = state
        .service
        .start_prepared_workflow_until_host_boundary(prepared, &state.host, false, |_, _| {})
        .await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn load_control_history<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlHistoryHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let events = ledger
        .load_events(&run_id)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlHistoryHttpResponse::new(
        run_id.as_str().to_owned(),
        events,
    )))
}

async fn load_control_summary<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlRunSummaryHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let summary = ledger
        .load_operator_summary(&run_id, now_unix_ms())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRunSummaryHttpResponse::new(summary)))
}

async fn load_control_recovery<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlRecoveryHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let recovery = ledger
        .load_recovery_snapshot(&run_id, now_unix_ms())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRecoveryHttpResponse::new(recovery)))
}

async fn load_control_diagnostics<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
) -> Result<Json<QianjiControlDiagnosticsHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let diagnostics = ledger
        .load_operator_diagnostics(&run_id, now_unix_ms())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlDiagnosticsHttpResponse::new(diagnostics)))
}

async fn apply_control_recovery<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(run_id): Path<String>,
    Json(request): Json<QianjiControlRecoveryApplyHttpRequest>,
) -> Result<Json<QianjiControlRecoveryApplyHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let ledger = control_ledger(&state)?;
    let hot_state = recovery_hot_state(&state)?;
    let run_id = parse_control_run_id(run_id)?;
    let occurred_at_ms = request.occurred_at_ms;
    let priority = request.priority;
    let attempt = recovery_attempt_from_request(request)?;
    let plan = ledger
        .load_recovery_plan(&run_id, occurred_at_ms)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    let application = apply_recovery_plan(
        ledger,
        hot_state,
        RecoveryLoopApplicationRequest::new(plan, attempt, occurred_at_ms, priority),
    )
    .await
    .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    let diagnostics = ledger
        .load_operator_diagnostics(&run_id, occurred_at_ms)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(Json(QianjiControlRecoveryApplyHttpResponse::new(
        application,
        diagnostics,
    )))
}

fn control_ledger<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
) -> Result<&dyn ControlLedger, QianjiBpmnWorkflowHttpError> {
    state.activity_evidence_ledger.as_deref().ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::service_unavailable(
            "control_ledger_unavailable",
            "qianji-server was not started with a control ledger",
        )
    })
}

fn recovery_hot_state<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
) -> Result<&dyn HotStateStore, QianjiBpmnWorkflowHttpError> {
    state.recovery_hot_state.as_deref().ok_or_else(|| {
        QianjiBpmnWorkflowHttpError::service_unavailable(
            "control_hot_state_unavailable",
            "qianji-server was not started with a recovery hot-state store",
        )
    })
}

fn parse_control_run_id(run_id: String) -> Result<RunId, QianjiBpmnWorkflowHttpError> {
    RunId::new(run_id).map_err(|_| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_control_run_id",
            "run_id must be a non-empty string",
        )
    })
}

fn recovery_attempt_from_request(
    request: QianjiControlRecoveryApplyHttpRequest,
) -> Result<RecoveryAttempt, QianjiBpmnWorkflowHttpError> {
    if request.reason.trim().is_empty() {
        return Err(QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_recovery_reason",
            "reason must be a non-empty string",
        ));
    }
    if request.max_attempts == 0 {
        return Err(QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_recovery_policy",
            "max_attempts must be greater than zero",
        ));
    }
    Ok(RecoveryAttempt {
        attempt: request.attempt,
        reason: request.reason,
        policy: RecoveryPolicy {
            max_attempts: request.max_attempts,
            backoff_ms: request.backoff_ms,
            require_human_approval: request.require_human_approval,
        },
    })
}

async fn resume_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_resume_request(instance_id);
    let prepared = state.service.prepare_resume_workflow(&request).await?;
    let report = state
        .service
        .resume_prepared_workflow_until_host_boundary(prepared, &state.host, false, |_, _| {})
        .await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn poll_workflow_events<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .poll_workflow_events(&request.into_event_poll_request(instance_id), &state.host)
        .await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn complete_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskCompleteHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_task_complete_request(instance_id);
    let report = complete_task_request(&state, request).await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn complete_workflow_tasks_batch<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskCompleteBatchHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let request = request.into_task_complete_batch_request(instance_id)?;
    let completions = request.completions.clone();
    let resume_request = request.workflow_resume_request();
    let prepared = state
        .service
        .prepare_resume_workflow(&resume_request)
        .await?;
    let pending_work = completions
        .iter()
        .map(|completion| matching_pending_work_for_completion(&prepared, completion))
        .collect::<Vec<_>>();
    let report = state
        .service
        .complete_prepared_workflow_task_batch_until_host_boundary(prepared, &request, &state.host)
        .await?;
    record_batch_activity_evidence(
        &state,
        &request.bpmn_path,
        &request.instance_id,
        pending_work,
        &completions,
    )?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn fail_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskFailHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowStatusHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let instance_id = QianjiBpmnWorkflowInstanceId::from(instance_id);
    let resume_request = request.workflow_resume_request(instance_id.clone());
    let prepared = state
        .service
        .prepare_resume_workflow(&resume_request)
        .await?;
    let pending_work =
        matching_pending_work_for_failure(&prepared, &request.failure).ok_or_else(|| {
            QianjiBpmnWorkflowHttpError::bad_request(
                "task_failure_not_pending",
                "failure identity does not match pending host work",
            )
        })?;
    record_failure_activity_evidence_for_pending(
        &state,
        &request.bpmn_path,
        &instance_id,
        &pending_work,
        &request.failure,
    )?;
    let report = state
        .service
        .load_workflow_status(&QianjiBpmnWorkflowStatusRequest {
            instance_id,
            checkpoint_backend: resume_request.checkpoint_backend,
        })
        .await?;
    Ok(Json(QianjiBpmnWorkflowStatusHttpResponse::from_report(
        &report,
    )))
}

async fn complete_task_request<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    request: QianjiBpmnWorkflowTaskCompleteRequest,
) -> Result<QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let resume_request = request.workflow_resume_request();
    let prepared = state
        .service
        .prepare_resume_workflow(&resume_request)
        .await?;
    let pending_work = matching_pending_work_for_completion(&prepared, &request.completion);
    let completion = request.completion.clone();
    let bpmn_path = request.bpmn_path.clone();
    let instance_id = request.instance_id.clone();
    let report = state
        .service
        .complete_prepared_workflow_task_until_host_boundary(prepared, &request, &state.host)
        .await?;
    record_single_activity_evidence(state, &bpmn_path, &instance_id, pending_work, &completion)?;
    Ok(report)
}

fn record_failure_activity_evidence_for_pending<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    bpmn_path: &std::path::Path,
    instance_id: &QianjiBpmnWorkflowInstanceId,
    pending_work: &xiuxian_qianji_bpmn_engine::PendingHostWork,
    failure: &QianjiBpmnWorkflowTaskFailureHttpPayload,
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let Some(ledger) = state.activity_evidence_ledger.as_deref() else {
        return Ok(());
    };
    record_failure_activity_evidence(
        ledger,
        QianjiBpmnWorkflowFailureActivityEvidenceInput {
            instance_id,
            bpmn_path,
            pending_work,
            failure,
            now_ms: now_unix_ms(),
        },
    )
}

fn record_single_activity_evidence<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    bpmn_path: &std::path::Path,
    instance_id: &QianjiBpmnWorkflowInstanceId,
    pending_work: Option<xiuxian_qianji_bpmn_engine::PendingHostWork>,
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    record_batch_activity_evidence(
        state,
        bpmn_path,
        instance_id,
        vec![pending_work],
        std::slice::from_ref(completion),
    )
}

fn record_batch_activity_evidence<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    bpmn_path: &std::path::Path,
    instance_id: &QianjiBpmnWorkflowInstanceId,
    pending_work: Vec<Option<xiuxian_qianji_bpmn_engine::PendingHostWork>>,
    completions: &[QianjiBpmnWorkflowTaskCompletionPayload],
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let Some(ledger) = state.activity_evidence_ledger.as_deref() else {
        return Ok(());
    };
    let now_ms = now_unix_ms();
    for (index, (pending_work, completion)) in
        pending_work.into_iter().zip(completions.iter()).enumerate()
    {
        let Some(pending_work) = pending_work else {
            continue;
        };
        let offset = u64::try_from(index).unwrap_or(u64::MAX).saturating_mul(3);
        record_completion_activity_evidence(
            ledger,
            QianjiBpmnWorkflowCompletionActivityEvidenceInput {
                instance_id,
                bpmn_path,
                pending_work: &pending_work,
                completion,
                now_ms: now_ms.saturating_add(offset),
            },
        )?;
    }
    Ok(())
}

async fn claim_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskClaimHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowTaskClaimHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .claim_workflow_task(&request.into_task_claim_request(instance_id))
        .await?;
    Ok(Json(QianjiBpmnWorkflowTaskClaimHttpResponse::from_report(
        &report,
    )))
}

async fn release_workflow_task<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowTaskReleaseHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowTaskReleaseHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .release_workflow_task(&request.into_task_release_request(instance_id))
        .await?;
    Ok(Json(
        QianjiBpmnWorkflowTaskReleaseHttpResponse::from_report(&report),
    ))
}

async fn load_workflow_status<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Query(query): Query<QianjiBpmnWorkflowStatusHttpQuery>,
) -> Result<Json<QianjiBpmnWorkflowStatusHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .load_workflow_status(&query.into_status_request(instance_id)?)
        .await?;
    Ok(Json(QianjiBpmnWorkflowStatusHttpResponse::from_report(
        &report,
    )))
}

async fn cancel_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowCancelHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .cancel_workflow(&request.into_cancel_request(instance_id))
        .await?;
    Ok(Json(QianjiBpmnWorkflowCancelHttpResponse::from_report(
        &report,
    )))
}
