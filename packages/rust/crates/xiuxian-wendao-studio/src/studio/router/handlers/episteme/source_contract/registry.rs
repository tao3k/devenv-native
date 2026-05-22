//! Ontology registry read-model Gateway admission.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "julia")]
use arrow::array::Array;
use axum::{Json, extract::State};
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::build_episteme_wendaograph_quality_request_batches;
use xiuxian_wendao::episteme::{
    EpistemeReadModelMaterialization,
    admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    WendaoGraphOntologyReadModelQualityRoundtrip,
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
};

use crate::studio::router::handlers::episteme::source_contract_support::{
    map_episteme_source_contract_error, resolve_episteme_root,
};
#[cfg(feature = "julia")]
use crate::studio::router::load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml;
use crate::studio::router::{GatewayState, StudioApiError};

use super::model::{
    EpistemeOntologyRegistryQualityProofGatewayReport,
    EpistemeOntologyRegistryQualityProofModeRequest,
    EpistemeOntologyRegistryReadModelGatewayReport,
    EpistemeOntologyRegistryReadModelGatewayRequest, EpistemeReadModelRowCountsGatewayReport,
    EpistemeReadModelTableGatewayReport,
};

const ONTOLOGY_REGISTRY_READ_MODEL_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_registry_read_model_admission.v1";
const ADMITTED_STATUS: &str = "admitted";
#[cfg(feature = "julia")]
const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV: &str =
    "WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL";
#[cfg(feature = "julia")]
const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_TIMEOUT_SECONDS_ENV: &str =
    "WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_TIMEOUT_SECONDS";

/// Admit an ontology registry snapshot and summarize its semantic read-model
/// batches through the Studio Gateway.
///
/// # Errors
///
/// Returns `BAD_REQUEST` for invalid request fields or rejected registry
/// snapshots, and `INTERNAL_SERVER_ERROR` for unexpected materialization
/// failures.
pub(crate) async fn admit_episteme_ontology_registry_read_model(
    State(state): State<Arc<GatewayState>>,
    Json(request): Json<EpistemeOntologyRegistryReadModelGatewayRequest>,
) -> Result<Json<EpistemeOntologyRegistryReadModelGatewayReport>, StudioApiError> {
    let report = admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof(
        state.studio.project_root.as_path(),
        state.studio.config_root.as_path(),
        &request,
    )
    .await?;
    Ok(Json(report))
}

#[cfg(test)]
pub(crate) fn admit_episteme_ontology_registry_read_model_from_payload(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeOntologyRegistryReadModelGatewayRequest,
) -> Result<EpistemeOntologyRegistryReadModelGatewayReport, StudioApiError> {
    let materialization = ontology_registry_read_model_materialization_from_payload(
        project_root,
        config_root,
        request,
    )?;
    Ok(ontology_registry_read_model_report(&materialization, None))
}

#[cfg(feature = "julia")]
pub(crate) async fn admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeOntologyRegistryReadModelGatewayRequest,
) -> Result<EpistemeOntologyRegistryReadModelGatewayReport, StudioApiError> {
    let materialization = ontology_registry_read_model_materialization_from_payload(
        project_root,
        config_root,
        request,
    )?;
    let quality_proof = ontology_registry_quality_proof_report(
        config_root,
        request
            .quality_proof_mode
            .unwrap_or(EpistemeOntologyRegistryQualityProofModeRequest::Disabled),
        &materialization,
    )
    .await?;
    Ok(ontology_registry_read_model_report(
        &materialization,
        quality_proof,
    ))
}

#[cfg(not(feature = "julia"))]
pub(crate) fn admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeOntologyRegistryReadModelGatewayRequest,
) -> std::future::Ready<Result<EpistemeOntologyRegistryReadModelGatewayReport, StudioApiError>> {
    let materialization = ontology_registry_read_model_materialization_from_payload(
        project_root,
        config_root,
        request,
    );
    std::future::ready(materialization.map(|materialization| {
        let quality_proof = ontology_registry_quality_proof_report(
            config_root,
            request
                .quality_proof_mode
                .unwrap_or(EpistemeOntologyRegistryQualityProofModeRequest::Disabled),
            &materialization,
        );
        ontology_registry_read_model_report(&materialization, quality_proof)
    }))
}

fn ontology_registry_read_model_materialization_from_payload(
    project_root: &Path,
    config_root: &Path,
    request: &EpistemeOntologyRegistryReadModelGatewayRequest,
) -> Result<EpistemeReadModelMaterialization, StudioApiError> {
    let episteme_root = resolve_episteme_root(project_root, config_root, request)?;
    admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed(&episteme_root)
        .map_err(map_episteme_source_contract_error)
}

fn ontology_registry_read_model_report(
    materialization: &EpistemeReadModelMaterialization,
    quality_proof: Option<EpistemeOntologyRegistryQualityProofGatewayReport>,
) -> EpistemeOntologyRegistryReadModelGatewayReport {
    let [
        semantic_objects,
        semantic_relations,
        semantic_projection_state,
    ] = materialization.row_counts();
    EpistemeOntologyRegistryReadModelGatewayReport {
        schema_version: ONTOLOGY_REGISTRY_READ_MODEL_SCHEMA_VERSION,
        status: ADMITTED_STATUS,
        source_revision: materialization.source_revision.clone(),
        row_counts: EpistemeReadModelRowCountsGatewayReport {
            objects: semantic_objects,
            relations: semantic_relations,
            projection_state: semantic_projection_state,
        },
        tables: materialization
            .tables
            .iter()
            .map(|table| EpistemeReadModelTableGatewayReport {
                table_name: table.table_name(),
                row_count: table.row_count(),
            })
            .collect(),
        quality_proof,
    }
}

#[cfg(feature = "julia")]
async fn ontology_registry_quality_proof_report(
    config_root: &Path,
    mode: EpistemeOntologyRegistryQualityProofModeRequest,
    materialization: &EpistemeReadModelMaterialization,
) -> Result<Option<EpistemeOntologyRegistryQualityProofGatewayReport>, StudioApiError> {
    match mode {
        EpistemeOntologyRegistryQualityProofModeRequest::Disabled => Ok(None),
        EpistemeOntologyRegistryQualityProofModeRequest::IfConfigured => {
            ontology_registry_quality_proof_report_if_configured(config_root, mode, materialization)
                .await
                .map(Some)
        }
    }
}

#[cfg(not(feature = "julia"))]
fn ontology_registry_quality_proof_report(
    config_root: &Path,
    mode: EpistemeOntologyRegistryQualityProofModeRequest,
    materialization: &EpistemeReadModelMaterialization,
) -> Option<EpistemeOntologyRegistryQualityProofGatewayReport> {
    match mode {
        EpistemeOntologyRegistryQualityProofModeRequest::Disabled => None,
        EpistemeOntologyRegistryQualityProofModeRequest::IfConfigured => {
            Some(ontology_registry_quality_proof_report_if_configured(
                config_root,
                mode,
                materialization,
            ))
        }
    }
}

#[cfg(feature = "julia")]
async fn ontology_registry_quality_proof_report_if_configured(
    config_root: &Path,
    mode: EpistemeOntologyRegistryQualityProofModeRequest,
    materialization: &EpistemeReadModelMaterialization,
) -> Result<EpistemeOntologyRegistryQualityProofGatewayReport, StudioApiError> {
    let proof_request = ontology_registry_quality_proof_request(materialization)?;
    let Some(endpoint) = ontology_registry_quality_proof_endpoint(config_root)? else {
        return Ok(ontology_registry_quality_proof_summary(
            QualityProofSummaryInput {
                mode,
                status: "not-configured",
                row_counts: proof_request.row_counts,
                payload_byte_sizes: proof_request.payload_byte_sizes,
                response_batch_count: None,
                response_row_count: None,
                selected_transport: None,
                response_status_counts: None,
            },
        ));
    };

    let binding = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url: endpoint.base_url,
            health_route: None,
            timeout_secs: endpoint.timeout_seconds,
            max_in_flight_requests: endpoint.max_in_flight_requests,
        },
    )
    .map_err(|error| {
        StudioApiError::internal(
            "EPISTEME_REGISTRY_WENDAOGRAPH_PROOF_BINDING_FAILED",
            "failed to build registry snapshot WendaoGraph proof binding",
            Some(error),
        )
    })?;
    let Some(roundtrip) = roundtrip_wendaograph_ontology_read_model_quality_with_binding(
        &binding,
        &proof_request.quality_batches,
    )
    .await
    .map_err(|error| {
        StudioApiError::internal(
            "EPISTEME_REGISTRY_WENDAOGRAPH_PROOF_ROUNDTRIP_FAILED",
            "registry snapshot WendaoGraph proof roundtrip failed",
            Some(error.error),
        )
    })?
    else {
        return Ok(ontology_registry_quality_proof_summary(
            QualityProofSummaryInput {
                mode,
                status: "transport-not-negotiated",
                row_counts: proof_request.row_counts,
                payload_byte_sizes: proof_request.payload_byte_sizes,
                response_batch_count: None,
                response_row_count: None,
                selected_transport: None,
                response_status_counts: None,
            },
        ));
    };
    let response_summary = ontology_registry_quality_response_summary(&roundtrip)?;
    Ok(ontology_registry_quality_proof_summary(
        QualityProofSummaryInput {
            mode,
            status: response_summary.status,
            row_counts: proof_request.row_counts,
            payload_byte_sizes: proof_request.payload_byte_sizes,
            response_batch_count: Some(response_summary.batch_count),
            response_row_count: Some(response_summary.row_count),
            selected_transport: Some(response_summary.selected_transport),
            response_status_counts: Some(response_summary.status_counts),
        },
    ))
}

#[cfg(feature = "julia")]
struct QualityProofRequest {
    quality_batches: WendaoGraphOntologyReadModelQualityRequestBatches,
    row_counts: [usize; 3],
    payload_byte_sizes: Vec<usize>,
}

#[cfg(feature = "julia")]
fn ontology_registry_quality_proof_request(
    materialization: &EpistemeReadModelMaterialization,
) -> Result<QualityProofRequest, StudioApiError> {
    let quality_batches = build_episteme_wendaograph_quality_request_batches(materialization)
        .map_err(map_episteme_source_contract_error)?;
    let row_counts = quality_batches.row_counts();
    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&quality_batches)
        .map_err(|error| {
            StudioApiError::internal(
                "EPISTEME_REGISTRY_WENDAOGRAPH_PROOF_PACKAGING_FAILED",
                "failed to package registry snapshot quality proof request",
                Some(error),
            )
        })?;
    Ok(QualityProofRequest {
        quality_batches,
        row_counts,
        payload_byte_sizes: request.payload_byte_sizes().to_vec(),
    })
}

#[cfg(feature = "julia")]
struct QualityProofEndpoint {
    base_url: String,
    timeout_seconds: Option<u64>,
    max_in_flight_requests: Option<u64>,
}

#[cfg(feature = "julia")]
fn ontology_registry_quality_proof_endpoint(
    config_root: &Path,
) -> Result<Option<QualityProofEndpoint>, StudioApiError> {
    let config =
        load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml(config_root);
    let Some(base_url) = config
        .as_ref()
        .map(|entry| entry.base_url.clone())
        .or_else(|| optional_env(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV))
    else {
        return Ok(None);
    };
    Ok(Some(QualityProofEndpoint {
        base_url,
        timeout_seconds: config.as_ref().and_then(|entry| entry.timeout_seconds).or(
            optional_u64_env(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_TIMEOUT_SECONDS_ENV)?,
        ),
        max_in_flight_requests: config
            .as_ref()
            .and_then(|entry| entry.max_in_flight_requests)
            .or(Some(1)),
    }))
}

#[cfg(feature = "julia")]
struct QualityProofResponseSummary {
    status: &'static str,
    batch_count: usize,
    row_count: usize,
    selected_transport: String,
    status_counts: BTreeMap<String, usize>,
}

#[cfg(feature = "julia")]
fn ontology_registry_quality_response_summary(
    roundtrip: &WendaoGraphOntologyReadModelQualityRoundtrip,
) -> Result<QualityProofResponseSummary, StudioApiError> {
    let status_counts = ontology_registry_quality_response_status_counts(
        &roundtrip.response_batches,
    )
    .map_err(|error| {
        StudioApiError::internal(
            "EPISTEME_REGISTRY_WENDAOGRAPH_PROOF_RESPONSE_INVALID",
            "registry snapshot WendaoGraph proof response was not a valid quality response",
            Some(error),
        )
    })?;
    let status = if status_counts
        .iter()
        .any(|(status, count)| *count > 0 && (status == "fail" || status == "error"))
    {
        "failed"
    } else {
        "passed"
    };
    Ok(QualityProofResponseSummary {
        status,
        batch_count: roundtrip.response_batches.len(),
        row_count: roundtrip
            .response_batches
            .iter()
            .map(arrow::record_batch::RecordBatch::num_rows)
            .sum(),
        selected_transport: format!("{:?}", roundtrip.selection.selected_transport),
        status_counts,
    })
}

#[cfg(not(feature = "julia"))]
fn ontology_registry_quality_proof_report_if_configured(
    _config_root: &Path,
    mode: EpistemeOntologyRegistryQualityProofModeRequest,
    materialization: &EpistemeReadModelMaterialization,
) -> EpistemeOntologyRegistryQualityProofGatewayReport {
    ontology_registry_quality_proof_summary(QualityProofSummaryInput {
        mode,
        status: "not-enabled",
        row_counts: materialization.row_counts(),
        payload_byte_sizes: Vec::new(),
        response_batch_count: None,
        response_row_count: None,
        selected_transport: None,
        response_status_counts: None,
    })
}

#[cfg(feature = "julia")]
fn ontology_registry_quality_response_status_counts(
    batches: &[arrow::record_batch::RecordBatch],
) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for batch in batches {
        let status_index = batch
            .schema()
            .index_of("status")
            .map_err(|error| format!("missing status column: {error}"))?;
        let statuses = batch
            .column(status_index)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .ok_or_else(|| "status column must be Utf8".to_string())?;
        for row_index in 0..statuses.len() {
            if statuses.is_null(row_index) {
                return Err(format!("status row {row_index} must not be null"));
            }
            *counts
                .entry(statuses.value(row_index).to_string())
                .or_default() += 1;
        }
    }
    if counts.is_empty() {
        return Err("quality response must include at least one status row".to_string());
    }
    Ok(counts)
}

struct QualityProofSummaryInput {
    mode: EpistemeOntologyRegistryQualityProofModeRequest,
    status: &'static str,
    row_counts: [usize; 3],
    payload_byte_sizes: Vec<usize>,
    response_batch_count: Option<usize>,
    response_row_count: Option<usize>,
    selected_transport: Option<String>,
    response_status_counts: Option<BTreeMap<String, usize>>,
}

fn ontology_registry_quality_proof_summary(
    input: QualityProofSummaryInput,
) -> EpistemeOntologyRegistryQualityProofGatewayReport {
    EpistemeOntologyRegistryQualityProofGatewayReport {
        mode: input.mode.as_str(),
        status: input.status,
        request_row_counts: EpistemeReadModelRowCountsGatewayReport {
            objects: input.row_counts[0],
            relations: input.row_counts[1],
            projection_state: input.row_counts[2],
        },
        payload_byte_sizes: input.payload_byte_sizes,
        response_batch_count: input.response_batch_count,
        response_row_count: input.response_row_count,
        selected_transport: input.selected_transport,
        response_status_counts: input.response_status_counts,
    }
}

#[cfg(feature = "julia")]
fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(feature = "julia")]
fn optional_u64_env(name: &str) -> Result<Option<u64>, StudioApiError> {
    let Some(value) = optional_env(name) else {
        return Ok(None);
    };
    value.parse::<u64>().map(Some).map_err(|error| {
        StudioApiError::bad_request(
            "EPISTEME_REGISTRY_WENDAOGRAPH_PROOF_CONFIG_INVALID",
            format!("invalid `{name}` value `{value}`: {error}"),
        )
    })
}
