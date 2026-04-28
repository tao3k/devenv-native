use super::api::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowHttpError, QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskClaimHttpResponse, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpRequest, QianjiBpmnWorkflowTaskReleaseHttpResponse,
};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use qianji_bpmn_engine::BpmnHostBridge;

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
            "/workflows/{instance_id}/tasks/claim",
            post(claim_workflow_task::<H>),
        )
        .route(
            "/workflows/{instance_id}/tasks/release",
            post(release_workflow_task::<H>),
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
    let report = state
        .service
        .start_workflow(&request.into_control_request(), &state.host)
        .await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
}

async fn resume_workflow<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Path(instance_id): Path<String>,
    Json(request): Json<QianjiBpmnWorkflowActionHttpRequest>,
) -> Result<Json<QianjiBpmnWorkflowRunHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let report = state
        .service
        .resume_workflow(&request.into_resume_request(instance_id), &state.host)
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
    let report = state
        .service
        .complete_workflow_task(
            &request.into_task_complete_request(instance_id),
            &state.host,
        )
        .await?;
    Ok(Json(QianjiBpmnWorkflowRunHttpResponse::from_start_report(
        &report,
    )))
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
