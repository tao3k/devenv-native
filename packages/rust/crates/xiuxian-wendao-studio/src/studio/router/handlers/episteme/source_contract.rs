//! Episteme Gateway admission handlers.

use std::path::Path;
use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Deserialize;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceReadReport, EpistemeEvidenceReadRequest, EpistemeEvidenceReadValidationMode,
    EpistemeEvidenceSelectionPlanRequest, EpistemeEvidenceSelectionValidationMode,
    EpistemeEvidenceSelectionWriteReport, EpistemeRunPlanRequest, EpistemeRunPlanWriteReport,
    read_episteme_evidence, read_episteme_evidence_selection_file_ids,
    write_episteme_evidence_selection_plan, write_episteme_extraction_run_plan,
};

use super::common::{
    EpistemeRootRequest, load_runtime_config, map_episteme_source_contract_error,
    resolve_corpus_root, resolve_episteme_root, resolve_run_root, trimmed_optional,
    trimmed_required,
};
use crate::studio::router::{GatewayState, StudioApiError};

/// Operational endpoint for episteme source-contract extraction run-plan admission.
pub(crate) const EPISTEME_SOURCE_CONTRACT_RUN_PLAN_ROUTE: &str =
    "/api/episteme/source-contract/extraction-run-plan";
/// Operational endpoint for targeted episteme source-contract evidence reads.
pub(crate) const EPISTEME_EVIDENCE_READ_ROUTE: &str = "/api/episteme/evidence/read";
/// Operational endpoint for episteme source-contract evidence selection plans.
pub(crate) const EPISTEME_EVIDENCE_SELECTION_PLAN_ROUTE: &str =
    "/api/episteme/evidence/selection-plan";

const DEFAULT_LIMIT: usize = 12;
const DEFAULT_MAX_PREVIEW_BYTES: usize = 8192;
const DEFAULT_SELECTION_REASON: &str = "manual_or_agent_selected";
const DEFAULT_RUN_ROOT: &str = "runs/extraction";
const DEFAULT_SELECTION_ROOT: &str = "runs/evidence-selection";

/// Request body for episteme source-contract run-plan admission.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeRunPlanAdmissionRequest {
    /// Episteme repository root.
    pub(crate) episteme_root: Option<String>,
    /// Episteme registry id from `wendao.toml`.
    pub(crate) episteme_registry_id: Option<String>,
    /// Corpus root. Falls back to the env var named by episteme config.
    pub(crate) corpus_root: Option<String>,
    /// Optional run artifact root. Defaults to `<epistemeRoot>/runs/extraction`.
    pub(crate) run_root: Option<String>,
    /// Evidence selection run id used to constrain extraction planning.
    pub(crate) selection_run_id: Option<String>,
    /// Optional evidence selection root. Defaults to `<epistemeRoot>/runs/evidence-selection`.
    pub(crate) selection_root: Option<String>,
    /// Safe ASCII run id.
    pub(crate) run_id: String,
    /// Optional extraction route filter.
    pub(crate) route: Option<String>,
    /// Optional category filter.
    pub(crate) category: Option<String>,
    /// Optional selected queue row limit.
    pub(crate) limit: Option<usize>,
}

/// Request body for targeted episteme source-contract evidence reads.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeEvidenceReadGatewayRequest {
    /// Episteme repository root.
    pub(crate) episteme_root: Option<String>,
    /// Episteme registry id from `wendao.toml`.
    pub(crate) episteme_registry_id: Option<String>,
    /// Corpus root. Falls back to the env var named by episteme config.
    pub(crate) corpus_root: Option<String>,
    /// Source-contract file id to read.
    pub(crate) file_id: String,
    /// Maximum bytes to include for supported text previews.
    pub(crate) max_preview_bytes: Option<usize>,
    /// Evidence read validation policy.
    pub(crate) validation_mode: Option<EpistemeEvidenceReadValidationModeRequest>,
}

/// JSON validation mode accepted by the Gateway evidence read endpoint.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum EpistemeEvidenceReadValidationModeRequest {
    /// Validate manifest and file metadata without hashing file contents.
    MetadataOnly,
    /// Run full source-contract validation, including sha256 drift checks.
    FullHash,
}

impl From<EpistemeEvidenceReadValidationModeRequest> for EpistemeEvidenceReadValidationMode {
    fn from(value: EpistemeEvidenceReadValidationModeRequest) -> Self {
        match value {
            EpistemeEvidenceReadValidationModeRequest::MetadataOnly => Self::MetadataOnly,
            EpistemeEvidenceReadValidationModeRequest::FullHash => Self::FullHash,
        }
    }
}

/// Request body for episteme source-contract evidence selection-plan writing.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeEvidenceSelectionPlanGatewayRequest {
    /// Episteme repository root.
    pub(crate) episteme_root: Option<String>,
    /// Episteme registry id from `wendao.toml`.
    pub(crate) episteme_registry_id: Option<String>,
    /// Corpus root. Falls back to the env var named by episteme config.
    pub(crate) corpus_root: Option<String>,
    /// Optional run artifact root. Defaults to `<epistemeRoot>/runs/evidence-selection`.
    pub(crate) run_root: Option<String>,
    /// Safe ASCII run id.
    pub(crate) run_id: String,
    /// Source-contract file ids selected for downstream evidence work.
    pub(crate) file_ids: Vec<String>,
    /// Run-level reason recorded in the selection ledger.
    pub(crate) selection_reason: Option<String>,
    /// Evidence selection validation policy.
    pub(crate) validation_mode: Option<EpistemeEvidenceSelectionValidationModeRequest>,
}

/// JSON validation mode accepted by the Gateway selection-plan endpoint.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum EpistemeEvidenceSelectionValidationModeRequest {
    /// Validate manifest and file metadata without hashing file contents.
    MetadataOnly,
    /// Run full source-contract validation, including sha256 drift checks.
    FullHash,
}

impl From<EpistemeEvidenceSelectionValidationModeRequest>
    for EpistemeEvidenceSelectionValidationMode
{
    fn from(value: EpistemeEvidenceSelectionValidationModeRequest) -> Self {
        match value {
            EpistemeEvidenceSelectionValidationModeRequest::MetadataOnly => Self::MetadataOnly,
            EpistemeEvidenceSelectionValidationModeRequest::FullHash => Self::FullHash,
        }
    }
}

/// Plan a episteme source-contract extraction run through the Studio Gateway.
///
/// # Errors
///
/// Returns `BAD_REQUEST` for invalid request fields or invalid source
/// contracts, and `INTERNAL_SERVER_ERROR` for write failures after admission.
pub(crate) async fn plan_episteme_extraction_run(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<EpistemeRunPlanAdmissionRequest>,
) -> Result<Json<EpistemeRunPlanWriteReport>, StudioApiError> {
    plan_episteme_extraction_run_from_payload(
        state.studio.project_root.as_path(),
        state.studio.config_root.as_path(),
        &request,
    )
    .map(Json)
}

/// Read one targeted episteme source-contract evidence row through the Studio
/// Gateway.
///
/// # Errors
///
/// Returns `BAD_REQUEST` for invalid request fields or invalid source
/// contracts, and `INTERNAL_SERVER_ERROR` for unexpected read failures.
pub(crate) async fn read_episteme_evidence_source(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<EpistemeEvidenceReadGatewayRequest>,
) -> Result<Json<EpistemeEvidenceReadReport>, StudioApiError> {
    read_episteme_evidence_source_from_payload(
        state.studio.project_root.as_path(),
        state.studio.config_root.as_path(),
        &request,
    )
    .map(Json)
}

/// Write a evidence-only selection plan through the Studio Gateway.
///
/// # Errors
///
/// Returns `BAD_REQUEST` for invalid request fields or invalid source
/// contracts, and `INTERNAL_SERVER_ERROR` for write failures after admission.
pub(crate) async fn write_episteme_evidence_selection_plan_source(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<EpistemeEvidenceSelectionPlanGatewayRequest>,
) -> Result<Json<EpistemeEvidenceSelectionWriteReport>, StudioApiError> {
    write_episteme_evidence_selection_plan_from_payload(
        state.studio.project_root.as_path(),
        state.studio.config_root.as_path(),
        &request,
    )
    .map(Json)
}

pub(crate) fn plan_episteme_extraction_run_from_payload(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeRunPlanAdmissionRequest,
) -> Result<EpistemeRunPlanWriteReport, StudioApiError> {
    let episteme_root = resolve_episteme_root(project_root, config_root, request)?;
    let runtime_config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        project_root,
        &episteme_root,
        request.corpus_root.as_deref(),
        runtime_config.as_ref(),
    )?;
    let run_root = resolve_run_root(
        project_root,
        request.run_root.as_deref(),
        runtime_config
            .as_ref()
            .and_then(|config| config.extraction_runs.as_deref()),
        || episteme_root.join(DEFAULT_RUN_ROOT),
    );
    let run_id = trimmed_required(request.run_id.as_str(), "runId")?.to_string();
    let route = trimmed_optional(request.route.as_deref());
    let category = trimmed_optional(request.category.as_deref());
    let limit = request.limit.unwrap_or(DEFAULT_LIMIT);

    let mut plan_request = EpistemeRunPlanRequest::new(&episteme_root, corpus_root, run_id);
    if let Some(route) = route {
        plan_request = plan_request.with_route(route);
    }
    if let Some(category) = category {
        plan_request = plan_request.with_category(category);
    }
    plan_request = plan_request.with_limit(limit);
    if let Some(selection_run_id) = trimmed_optional(request.selection_run_id.as_deref()) {
        let selection_root = resolve_run_root(
            project_root,
            request.selection_root.as_deref(),
            runtime_config
                .as_ref()
                .and_then(|config| config.evidence_selection_runs.as_deref()),
            || episteme_root.join(DEFAULT_SELECTION_ROOT),
        );
        let selection_tsv_path = selection_root.join(selection_run_id).join("selection.tsv");
        let selected_file_ids = read_episteme_evidence_selection_file_ids(selection_tsv_path)
            .map_err(map_episteme_source_contract_error)?;
        plan_request = plan_request.with_selected_file_ids(selected_file_ids);
    }

    write_episteme_extraction_run_plan(&plan_request, run_root)
        .map_err(map_episteme_source_contract_error)
}

pub(crate) fn read_episteme_evidence_source_from_payload(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeEvidenceReadGatewayRequest,
) -> Result<EpistemeEvidenceReadReport, StudioApiError> {
    let episteme_root = resolve_episteme_root(project_root, config_root, request)?;
    let runtime_config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        project_root,
        &episteme_root,
        request.corpus_root.as_deref(),
        runtime_config.as_ref(),
    )?;
    let file_id = trimmed_required(request.file_id.as_str(), "fileId")?.to_string();
    let validation_mode = request
        .validation_mode
        .map_or(EpistemeEvidenceReadValidationMode::MetadataOnly, Into::into);
    let read_request = EpistemeEvidenceReadRequest::new(&episteme_root, corpus_root, file_id)
        .with_max_preview_bytes(
            request
                .max_preview_bytes
                .unwrap_or(DEFAULT_MAX_PREVIEW_BYTES),
        )
        .with_validation_mode(validation_mode);

    read_episteme_evidence(&read_request).map_err(map_episteme_source_contract_error)
}

pub(crate) fn write_episteme_evidence_selection_plan_from_payload(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeEvidenceSelectionPlanGatewayRequest,
) -> Result<EpistemeEvidenceSelectionWriteReport, StudioApiError> {
    let episteme_root = resolve_episteme_root(project_root, config_root, request)?;
    let runtime_config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        project_root,
        &episteme_root,
        request.corpus_root.as_deref(),
        runtime_config.as_ref(),
    )?;
    let run_root = resolve_run_root(
        project_root,
        request.run_root.as_deref(),
        runtime_config
            .as_ref()
            .and_then(|config| config.evidence_selection_runs.as_deref()),
        || episteme_root.join(DEFAULT_SELECTION_ROOT),
    );
    let run_id = trimmed_required(request.run_id.as_str(), "runId")?.to_string();
    let selection_reason = trimmed_optional(request.selection_reason.as_deref())
        .unwrap_or_else(|| DEFAULT_SELECTION_REASON.to_string());
    let validation_mode = request.validation_mode.map_or(
        EpistemeEvidenceSelectionValidationMode::MetadataOnly,
        Into::into,
    );
    let write_request = EpistemeEvidenceSelectionPlanRequest::new(
        &episteme_root,
        corpus_root,
        run_id,
        request.file_ids.clone(),
    )
    .with_selection_reason(selection_reason)
    .with_validation_mode(validation_mode);

    write_episteme_evidence_selection_plan(&write_request, run_root)
        .map_err(map_episteme_source_contract_error)
}

impl EpistemeRootRequest for EpistemeRunPlanAdmissionRequest {
    fn episteme_root(&self) -> Option<&str> {
        self.episteme_root.as_deref()
    }

    fn episteme_registry_id(&self) -> Option<&str> {
        self.episteme_registry_id.as_deref()
    }
}

impl EpistemeRootRequest for EpistemeEvidenceReadGatewayRequest {
    fn episteme_root(&self) -> Option<&str> {
        self.episteme_root.as_deref()
    }

    fn episteme_registry_id(&self) -> Option<&str> {
        self.episteme_registry_id.as_deref()
    }
}

impl EpistemeRootRequest for EpistemeEvidenceSelectionPlanGatewayRequest {
    fn episteme_root(&self) -> Option<&str> {
        self.episteme_root.as_deref()
    }

    fn episteme_registry_id(&self) -> Option<&str> {
        self.episteme_registry_id.as_deref()
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/episteme/source_contract.rs"]
mod tests;
