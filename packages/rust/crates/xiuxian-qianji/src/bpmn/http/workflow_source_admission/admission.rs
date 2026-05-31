use super::compile::compile_markdown_workflow_source;
use super::contract::{MARKDOWN_MEDIA_TYPE, WorkflowSourceCompilation, WorkflowSourceCompileError};
use super::server_repair::{ServerRepairCompilerRequest, start_server_repair_workflow};
use crate::bpmn::http_transport::bpmn_source_admission::{
    admission_bad_request, admit_bpmn_source_request, sha256_digest,
};
use crate::bpmn::http_transport::error_api::QianjiBpmnWorkflowHttpError;
use crate::bpmn::http_transport::request_api::{
    QianjiControlBpmnSourceAdmissionHttpRequest, QianjiControlWorkflowSourceAdmissionHttpRequest,
    QianjiControlWorkflowSourceCompilerMode,
};
use crate::bpmn::http_transport::response_api::QianjiControlWorkflowSourceAdmissionHttpResponse;
use crate::bpmn::http_transport::source_authoring::QianjiControlWorkflowSourceAuthoringMediaType;
use crate::bpmn::http_transport::state::QianjiBpmnWorkflowHttpState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use xiuxian_qianji_bpmn_engine::BpmnHostBridge;

pub(in crate::bpmn::http_transport) async fn admit_control_workflow_source<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Json(request): Json<QianjiControlWorkflowSourceAdmissionHttpRequest>,
) -> Result<
    (
        StatusCode,
        Json<QianjiControlWorkflowSourceAdmissionHttpResponse>,
    ),
    QianjiBpmnWorkflowHttpError,
>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let authoring_media_type = normalize_authoring_media_type(&request.source_media_type)?;
    validate_workflow_source_request(&request)?;

    let authoring_source_sha256 = sha256_digest(request.source_text.as_bytes());
    if request.compiler_mode == QianjiControlWorkflowSourceCompilerMode::ServerRepair {
        let repair_request =
            ServerRepairCompilerRequest::from_admission_request(&request, &authoring_media_type);
        let repair_run = start_server_repair_workflow(
            &state,
            &request,
            &repair_request,
            &authoring_media_type,
            &authoring_source_sha256,
        )
        .await?;
        return Ok((
            StatusCode::ACCEPTED,
            Json(
                QianjiControlWorkflowSourceAdmissionHttpResponse::repair_started(
                    request.source_id,
                    request.process_id,
                    authoring_media_type.clone(),
                    authoring_source_sha256,
                    repair_request.compiler(),
                    repair_run,
                ),
            ),
        ));
    }

    let compilation = compile_workflow_source(&request)?;
    let admitted = admit_bpmn_source_request(
        &state,
        QianjiControlBpmnSourceAdmissionHttpRequest {
            source_id: request.source_id,
            process_id: request.process_id,
            bpmn_xml: compilation.bpmn_xml,
        },
    )?;
    Ok((
        StatusCode::OK,
        Json(QianjiControlWorkflowSourceAdmissionHttpResponse::new(
            admitted,
            authoring_media_type.clone(),
            authoring_source_sha256,
            compilation.compiler,
        )),
    ))
}

fn compile_workflow_source(
    request: &QianjiControlWorkflowSourceAdmissionHttpRequest,
) -> Result<WorkflowSourceCompilation, QianjiBpmnWorkflowHttpError> {
    match request.compiler_mode {
        QianjiControlWorkflowSourceCompilerMode::DeterministicMarkdownStep => {
            compile_markdown_workflow_source(request).map_err(compile_error)
        }
        QianjiControlWorkflowSourceCompilerMode::ServerRepair => unreachable!(
            "server_repair is handled before deterministic workflow-source compilation"
        ),
    }
}

fn compile_error(error: WorkflowSourceCompileError) -> QianjiBpmnWorkflowHttpError {
    match error {
        WorkflowSourceCompileError::MarkdownStepsMissing => admission_bad_request(
            "workflow_source_repair_required",
            "workflow source admission requires explicit `## Step N: Title` sections until the server-owned Skill.md/pi-agent repair compiler is enabled",
        ),
    }
}

fn normalize_authoring_media_type(
    media_type: &QianjiControlWorkflowSourceAuthoringMediaType,
) -> Result<QianjiControlWorkflowSourceAuthoringMediaType, QianjiBpmnWorkflowHttpError> {
    let normalized = media_type
        .as_str()
        .split_once(';')
        .map_or(media_type.as_str(), |(value, _)| value)
        .trim()
        .to_ascii_lowercase();
    if normalized == MARKDOWN_MEDIA_TYPE {
        return Ok(QianjiControlWorkflowSourceAuthoringMediaType::from_text_markdown());
    }
    Err(admission_bad_request(
        "workflow_source_media_type_unsupported",
        "workflow source admission currently supports source_media_type `text/markdown`",
    ))
}

fn validate_workflow_source_request(
    request: &QianjiControlWorkflowSourceAdmissionHttpRequest,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    if request.source_text.trim().is_empty() {
        return Err(admission_bad_request(
            "workflow_source_text_empty",
            "workflow source admission requires non-empty source_text",
        ));
    }
    if request.workflow_name.trim().is_empty() {
        return Err(admission_bad_request(
            "workflow_source_name_empty",
            "workflow source admission requires a non-empty workflow_name",
        ));
    }
    Ok(())
}
