use crate::bpmn::control::QianjiBpmnWorkflowControlError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// JSON error body emitted by BPMN workflow HTTP routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowHttpErrorBody {
    /// Stable machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

#[derive(Debug)]
pub(in crate::bpmn::http_transport) struct QianjiBpmnWorkflowHttpError {
    status: StatusCode,
    body: QianjiBpmnWorkflowHttpErrorBody,
}

impl QianjiBpmnWorkflowHttpError {
    pub(in crate::bpmn::http_transport) fn bad_request(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: QianjiBpmnWorkflowHttpErrorBody {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

impl From<QianjiBpmnWorkflowControlError> for QianjiBpmnWorkflowHttpError {
    fn from(error: QianjiBpmnWorkflowControlError) -> Self {
        let (status, code) = match error {
            QianjiBpmnWorkflowControlError::CheckpointMissing { .. } => {
                (StatusCode::NOT_FOUND, "checkpoint_missing")
            }
            QianjiBpmnWorkflowControlError::Io(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "workflow_control_io")
            }
            QianjiBpmnWorkflowControlError::Orchestration(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "workflow_orchestration_failed",
            ),
        };
        Self {
            status,
            body: QianjiBpmnWorkflowHttpErrorBody {
                code: code.to_string(),
                message: error.to_string(),
            },
        }
    }
}

impl IntoResponse for QianjiBpmnWorkflowHttpError {
    fn into_response(self) -> Response {
        (self.status, axum::Json(self.body)).into_response()
    }
}
