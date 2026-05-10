//! Owns the Studio studio router error surface.

use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::studio::types::ApiError;
use xiuxian_wendao::analyzers::RepoIntelligenceError;

/// Studio API error type.
#[derive(Debug, serde::Serialize, Clone)]
pub struct StudioApiError {
    #[serde(skip)]
    /// HTTP status returned for the error.
    pub status: StatusCode,
    /// Serialized API error payload.
    pub error: ApiError,
}

impl StudioApiError {
    /// Return the HTTP status associated with the error.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Return the stable error code carried by the payload.
    #[must_use]
    pub fn code(&self) -> &str {
        self.error.code.as_str()
    }

    /// Creates a bad request error.
    pub fn bad_request(code: &str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: ApiError {
                code: code.to_string(),
                message: message.into(),
                details: None,
            },
        }
    }

    /// Creates a not found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error: ApiError {
                code: "NOT_FOUND".to_string(),
                message: message.into(),
                details: None,
            },
        }
    }

    /// Creates an internal server error.
    pub fn internal(code: &str, message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: ApiError {
                code: code.to_string(),
                message: message.into(),
                details,
            },
        }
    }

    /// Creates a conflict error.
    pub fn conflict(code: &str, message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            error: ApiError {
                code: code.to_string(),
                message: message.into(),
                details,
            },
        }
    }

    /// Creates a standard index-not-ready error for search corpora.
    #[must_use]
    pub fn index_not_ready(corpus: &str) -> Self {
        Self::conflict(
            "INDEX_NOT_READY",
            format!("search corpus `{corpus}` has not published an index epoch yet"),
            Some(corpus.to_string()),
        )
    }
}

impl IntoResponse for StudioApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.error.clone())).into_response()
    }
}

/// Maps a `RepoIntelligenceError` to a `StudioApiError`.
#[must_use]
pub fn map_repo_intelligence_error(error: RepoIntelligenceError) -> StudioApiError {
    match error {
        RepoIntelligenceError::UnknownRepository { repo_id } => StudioApiError::bad_request(
            "UNKNOWN_REPOSITORY",
            format!("Repo Intelligence repository `{repo_id}` is not registered"),
        ),
        RepoIntelligenceError::MissingRepoIntelligencePlugins { repo_id } => {
            StudioApiError::bad_request(
                "MISSING_REQUIRED_PLUGIN",
                format!("repo `{repo_id}` does not configure any repo-intelligence plugins"),
            )
        }
        RepoIntelligenceError::MissingRequiredPlugin { repo_id, plugin_id } => {
            StudioApiError::bad_request(
                "MISSING_REQUIRED_PLUGIN",
                format!("repo `{repo_id}` requires plugin `{plugin_id}`"),
            )
        }
        RepoIntelligenceError::MissingPlugin { plugin_id } => StudioApiError::bad_request(
            "MISSING_PLUGIN",
            format!("repo intelligence plugin `{plugin_id}` is not registered"),
        ),
        RepoIntelligenceError::MissingRepositoryPath { repo_id } => StudioApiError::bad_request(
            "MISSING_REPOSITORY_PATH",
            format!("repo `{repo_id}` does not declare a local path"),
        ),
        RepoIntelligenceError::MissingRepositorySource { repo_id } => StudioApiError::bad_request(
            "MISSING_REPOSITORY_SOURCE",
            format!("repo `{repo_id}` must declare a local path or upstream url"),
        ),
        RepoIntelligenceError::InvalidRepositoryPath { path, reason, .. } => {
            StudioApiError::bad_request(
                "INVALID_REPOSITORY_PATH",
                format!("invalid repository path `{path}`: {reason}"),
            )
        }
        RepoIntelligenceError::UnsupportedRepositoryLayout { repo_id, message } => {
            StudioApiError::bad_request(
                "UNSUPPORTED_REPOSITORY_LAYOUT",
                format!("repo `{repo_id}` has unsupported layout: {message}"),
            )
        }
        RepoIntelligenceError::PendingRepositoryIndex { repo_id } => StudioApiError::conflict(
            "REPO_INDEX_PENDING",
            format!("repo `{repo_id}` index is still warming"),
            Some(repo_id),
        ),
        RepoIntelligenceError::UnknownProjectedPage { repo_id, page_id } => {
            StudioApiError::not_found(format!(
                "repo `{repo_id}` does not contain projected page `{page_id}`"
            ))
        }
        RepoIntelligenceError::UnknownProjectedGap { repo_id, gap_id } => {
            StudioApiError::not_found(format!(
                "repo `{repo_id}` does not contain projected gap `{gap_id}`"
            ))
        }
        RepoIntelligenceError::UnknownProjectedPageFamilyCluster {
            repo_id,
            page_id,
            kind,
        } => StudioApiError::not_found(format!(
            "repo `{repo_id}` does not contain projected page family `{kind:?}` in page `{page_id}`"
        )),
        RepoIntelligenceError::UnknownProjectedPageIndexNode {
            repo_id,
            page_id,
            node_id,
        } => StudioApiError::not_found(format!(
            "repo `{repo_id}` does not contain projected page-index node `{node_id}` in page `{page_id}`"
        )),
        RepoIntelligenceError::ConfigLoad { message } => {
            StudioApiError::bad_request("CONFIG_LOAD_FAILED", message)
        }
        RepoIntelligenceError::DuplicatePlugin { plugin_id } => StudioApiError::internal(
            "DUPLICATE_PLUGIN",
            "Repo intelligence plugin registry is inconsistent",
            Some(format!("duplicate plugin `{plugin_id}`")),
        ),
        RepoIntelligenceError::AnalysisFailed { message } => StudioApiError::internal(
            "REPO_INTELLIGENCE_FAILED",
            "Repo intelligence task failed",
            Some(message),
        ),
    }
}
