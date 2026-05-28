//! Server-owned BPMN source admission.

use super::error_api::QianjiBpmnWorkflowHttpError;
use super::request_api::QianjiControlBpmnSourceAdmissionHttpRequest;
use super::response_api::QianjiControlBpmnSourceAdmissionHttpResponse;
use super::state::QianjiBpmnWorkflowHttpState;
use crate::runtime_config::QianjiRuntimeEnv;
use axum::Json;
use axum::extract::State;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use xiuxian_qianji_bpmn_engine::{
    BpmnHostBridge, BpmnParseOptions, BpmnSourceFile, lint_bpmn_source, parse_bpmn_package,
};

pub(super) async fn admit_control_bpmn_source<H>(
    State(state): State<QianjiBpmnWorkflowHttpState<H>>,
    Json(request): Json<QianjiControlBpmnSourceAdmissionHttpRequest>,
) -> Result<Json<QianjiControlBpmnSourceAdmissionHttpResponse>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let source_id = normalize_source_id(&request.source_id)?;
    let process_id = request.process_id.clone();
    if process_id.as_str().trim().is_empty() {
        return Err(admission_bad_request(
            "bpmn_source_process_id_empty",
            "BPMN source admission requires a non-empty process_id",
        ));
    }
    if request.bpmn_xml.trim().is_empty() {
        return Err(admission_bad_request(
            "bpmn_source_xml_empty",
            "BPMN source admission requires non-empty bpmn_xml",
        ));
    }

    let source = BpmnSourceFile::new(format!("{source_id}.bpmn"), request.bpmn_xml.clone());
    let lint_report = lint_bpmn_source(&source);
    if !lint_report.ok {
        return Err(admission_bad_request(
            "bpmn_source_lint_failed",
            format!(
                "BPMN source '{source_id}' failed lint with {} blocking issue(s)",
                lint_report.issues.len()
            ),
        ));
    }

    let package = parse_bpmn_package(std::slice::from_ref(&source), &BpmnParseOptions::default())
        .map_err(|error| {
        admission_bad_request(
            "bpmn_source_parse_failed",
            format!("BPMN source '{source_id}' failed parse: {error}"),
        )
    })?;
    if package.find_process(process_id.as_str()).is_none() {
        return Err(admission_bad_request(
            "bpmn_source_process_missing",
            format!(
                "BPMN source '{source_id}' does not define process '{}'",
                process_id.as_str()
            ),
        ));
    }

    let source_sha256 = sha256_digest(request.bpmn_xml.as_bytes());
    let admitted_path = admitted_source_path(
        state.runtime_env.as_ref(),
        &source_id,
        source_sha256
            .strip_prefix("sha256:")
            .unwrap_or(source_sha256.as_str()),
    )?;
    if let Some(parent) = admitted_path.parent() {
        std::fs::create_dir_all(parent).map_err(admission_io_error)?;
    }
    std::fs::write(&admitted_path, request.bpmn_xml).map_err(admission_io_error)?;
    let source_ref = admitted_path.display().to_string();

    Ok(Json(QianjiControlBpmnSourceAdmissionHttpResponse::new(
        source_id,
        source_ref,
        process_id,
        source_sha256,
        lint_report.issues.len(),
    )))
}

fn normalize_source_id(source_id: &str) -> Result<String, QianjiBpmnWorkflowHttpError> {
    let normalized = source_id
        .trim()
        .chars()
        .map(|character| match character {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => character,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches(['_', '-', '.'])
        .to_string();
    if normalized.is_empty() {
        return Err(admission_bad_request(
            "bpmn_source_id_empty",
            "BPMN source admission requires a source_id with at least one filesystem-safe character",
        ));
    }
    Ok(normalized)
}

fn admitted_source_path(
    runtime_env: Option<&QianjiRuntimeEnv>,
    source_id: &str,
    hash_hex: &str,
) -> Result<PathBuf, QianjiBpmnWorkflowHttpError> {
    let hash_prefix = hash_hex
        .get(..16)
        .ok_or_else(|| admission_io_error("BPMN source hash is unexpectedly short"))?;
    Ok(admission_root(runtime_env).join(format!("{source_id}-{hash_prefix}.bpmn")))
}

fn admission_root(runtime_env: Option<&QianjiRuntimeEnv>) -> PathBuf {
    if let Some(project_root) = runtime_env.and_then(|runtime_env| runtime_env.prj_root.as_ref()) {
        return project_root.join(".cache/qianji/bpmn-sources");
    }
    if let Some(path) = std::env::var_os("PRJ_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return path.join("qianji/bpmn-sources");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".cache/qianji/bpmn-sources")
}

fn sha256_digest(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    format!("sha256:{digest:x}")
}

fn admission_bad_request(
    code: impl Into<String>,
    message: impl Into<String>,
) -> QianjiBpmnWorkflowHttpError {
    QianjiBpmnWorkflowHttpError::bad_request(code, message)
}

fn admission_io_error(error: impl std::fmt::Display) -> QianjiBpmnWorkflowHttpError {
    QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string())
}
