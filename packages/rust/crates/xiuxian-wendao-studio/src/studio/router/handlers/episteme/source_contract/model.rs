//! Episteme source-contract Gateway DTOs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use xiuxian_wendao::episteme::{
    EpistemeEvidenceReadValidationMode, EpistemeEvidenceSelectionValidationMode,
};

use crate::studio::router::handlers::episteme::source_contract_support::EpistemeRootRequest;

/// Operational endpoint for episteme source-contract extraction run-plan admission.
pub(crate) const EPISTEME_SOURCE_CONTRACT_RUN_PLAN_ROUTE: &str =
    "/api/episteme/source-contract/extraction-run-plan";
/// Operational endpoint for targeted episteme source-contract evidence reads.
pub(crate) const EPISTEME_EVIDENCE_READ_ROUTE: &str = "/api/episteme/evidence/read";
/// Operational endpoint for episteme source-contract evidence selection plans.
pub(crate) const EPISTEME_EVIDENCE_SELECTION_PLAN_ROUTE: &str =
    "/api/episteme/evidence/selection-plan";
/// Operational endpoint for ontology registry snapshot read-model admission.
pub(crate) const EPISTEME_ONTOLOGY_REGISTRY_READ_MODEL_ROUTE: &str =
    "/api/episteme/ontology-registry/read-model";

pub(crate) const DEFAULT_LIMIT: usize = 12;
pub(crate) const DEFAULT_MAX_PREVIEW_BYTES: usize = 8192;
pub(crate) const DEFAULT_SELECTION_REASON: &str = "manual_or_agent_selected";
pub(crate) const DEFAULT_RUN_ROOT: &str = "runs/extraction";
pub(crate) const DEFAULT_SELECTION_ROOT: &str = "runs/evidence-selection";
pub(crate) const QUALITY_PROOF_MODE_IF_CONFIGURED: &str = "if-configured";

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

/// Request body for ontology registry snapshot read-model admission.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeOntologyRegistryReadModelGatewayRequest {
    /// Episteme repository root.
    pub(crate) episteme_root: Option<String>,
    /// Episteme registry id from `wendao.toml`.
    pub(crate) episteme_registry_id: Option<String>,
    /// Optional live `WendaoGraph` quality proof mode.
    pub(crate) quality_proof_mode: Option<EpistemeOntologyRegistryQualityProofModeRequest>,
}

/// Optional `WendaoGraph` quality proof policy for registry snapshot admission.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum EpistemeOntologyRegistryQualityProofModeRequest {
    /// Keep admission local and do not package or send a quality proof request.
    Disabled,
    /// Package a proof request and run it only when an endpoint is configured.
    IfConfigured,
}

impl EpistemeOntologyRegistryQualityProofModeRequest {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::IfConfigured => QUALITY_PROOF_MODE_IF_CONFIGURED,
        }
    }
}

/// Gateway report for ontology registry snapshot read-model admission.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeOntologyRegistryReadModelGatewayReport {
    /// Report schema version.
    pub(crate) schema_version: &'static str,
    /// Admission status.
    pub(crate) status: &'static str,
    /// Deterministic source revision of the admitted registry snapshot.
    pub(crate) source_revision: String,
    /// Semantic read-model row counts.
    pub(crate) row_counts: EpistemeReadModelRowCountsGatewayReport,
    /// Per-table summaries in service order.
    pub(crate) tables: Vec<EpistemeReadModelTableGatewayReport>,
    /// Optional live `WendaoGraph` quality proof summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quality_proof: Option<EpistemeOntologyRegistryQualityProofGatewayReport>,
}

/// Semantic read-model row counts.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeReadModelRowCountsGatewayReport {
    /// `semantic_objects` row count.
    #[serde(rename = "semanticObjects")]
    pub(crate) objects: usize,
    /// `semantic_relations` row count.
    #[serde(rename = "semanticRelations")]
    pub(crate) relations: usize,
    /// `semantic_projection_state` row count.
    #[serde(rename = "semanticProjectionState")]
    pub(crate) projection_state: usize,
}

/// One semantic read-model table summary.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeReadModelTableGatewayReport {
    /// Table name.
    pub(crate) table_name: &'static str,
    /// Row count.
    pub(crate) row_count: usize,
}

/// Optional `WendaoGraph` quality proof summary.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeOntologyRegistryQualityProofGatewayReport {
    /// Requested proof mode.
    pub(crate) mode: &'static str,
    /// Proof execution status.
    pub(crate) status: &'static str,
    /// Semantic read-model rows sent or eligible to be sent.
    pub(crate) request_row_counts: EpistemeReadModelRowCountsGatewayReport,
    /// Arrow IPC payload byte sizes in service request order when packaging runs.
    pub(crate) payload_byte_sizes: Vec<usize>,
    /// Number of response batches returned by the live service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_batch_count: Option<usize>,
    /// Total response rows returned by the live service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_row_count: Option<usize>,
    /// Selected runtime transport, when negotiation reached a live service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selected_transport: Option<String>,
    /// Response quality status counts, when the live service returns rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_status_counts: Option<BTreeMap<String, usize>>,
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

impl EpistemeRootRequest for EpistemeOntologyRegistryReadModelGatewayRequest {
    fn episteme_root(&self) -> Option<&str> {
        self.episteme_root.as_deref()
    }

    fn episteme_registry_id(&self) -> Option<&str> {
        self.episteme_registry_id.as_deref()
    }
}
