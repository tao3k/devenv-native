use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use xiuxian_wendao::episteme::{
    EpistemeRegistryEntry, load_episteme_registry_entries, load_episteme_runtime_config,
    validate_episteme_registry_reference_graph,
};
use xiuxian_wendao_server::transport::{
    OntologyCandidateInspectionFlightRequest, OntologyCandidateInspectionFlightRouteProvider,
    OntologyCandidateInspectionFlightRouteResponse,
};
use xiuxian_wendao_sql::candidate_read_model::{
    CandidateReadModelDuckDbInspectionReport, CandidateReadModelDuckDbInspectionRequest,
    inspect_candidate_read_model_with_duckdb,
};

use crate::studio::GatewayState;
use crate::studio::router::load_episteme_registry_from_wendao_toml;

/// Studio-owned ontology candidate inspection provider for the Gateway Flight
/// service.
#[derive(Clone)]
pub(crate) struct StudioOntologyCandidateInspectionFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioOntologyCandidateInspectionFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioOntologyCandidateInspectionFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioOntologyCandidateInspectionFlightRouteProvider")
            .field(
                "project_root",
                &self
                    .state
                    .studio
                    .project_root
                    .as_path()
                    .display()
                    .to_string(),
            )
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl OntologyCandidateInspectionFlightRouteProvider
    for StudioOntologyCandidateInspectionFlightRouteProvider
{
    async fn ontology_candidate_inspection_batch(
        &self,
        request: &OntologyCandidateInspectionFlightRequest,
    ) -> Result<OntologyCandidateInspectionFlightRouteResponse, String> {
        let run_dir = resolve_candidate_run_dir(
            self.state.studio.project_root.as_path(),
            self.state.studio.config_root.as_path(),
            request.episteme_registry_id.as_str(),
            request.run_id.as_str(),
        )?;
        let inspection_request =
            CandidateReadModelDuckDbInspectionRequest::from_candidate_run_dir(run_dir);
        let report = inspect_candidate_read_model_with_duckdb(&inspection_request)?;
        let batch = candidate_inspection_report_batch(&report)?;
        let app_metadata = serde_json::to_vec(&report)
            .map_err(|error| format!("failed to encode candidate inspection metadata: {error}"))?;
        Ok(
            OntologyCandidateInspectionFlightRouteResponse::from_batches(vec![batch])
                .with_app_metadata(app_metadata),
        )
    }
}

fn resolve_candidate_run_dir(
    project_root: &Path,
    config_root: &Path,
    episteme_registry_id: &str,
    run_id: &str,
) -> Result<PathBuf, String> {
    let episteme_root =
        resolve_episteme_registry_root(project_root, config_root, episteme_registry_id)?;
    let runtime_config = load_episteme_runtime_config(episteme_root.as_path())
        .map_err(|error| format!("failed to load Episteme runtime config: {error}"))?;
    let run_root = runtime_config
        .and_then(|config| config.ontology_generation_runs)
        .unwrap_or_else(|| episteme_root.join("runs/ontology-generation"));
    Ok(run_root.join(run_id))
}

fn resolve_episteme_registry_root(
    project_root: &Path,
    config_root: &Path,
    registry_id: &str,
) -> Result<PathBuf, String> {
    let entries = load_episteme_registry_from_wendao_toml(config_root)
        .map_err(|error| format!("failed to load Episteme registry config: {error}"))?;
    let Some(entry) = find_episteme_registry_entry(entries.as_slice(), registry_id) else {
        return Err(format!(
            "Episteme registry `{registry_id}` is not configured"
        ));
    };
    if !entry.enabled {
        return Err(format!("Episteme registry `{registry_id}` is disabled"));
    }
    let receipt = load_episteme_registry_entries(entries.as_slice(), project_root)
        .map_err(|error| format!("failed to load Episteme registry `{registry_id}`: {error}"))?;
    validate_episteme_registry_reference_graph(&receipt)
        .map_err(|error| format!("Episteme registry `{registry_id}` is invalid: {error}"))?;
    receipt
        .entries
        .into_iter()
        .find(|entry| entry.id == registry_id)
        .map(|entry| entry.episteme_root)
        .ok_or_else(|| format!("Episteme registry `{registry_id}` did not load an episteme root"))
}

fn find_episteme_registry_entry<'a>(
    entries: &'a [EpistemeRegistryEntry],
    registry_id: &str,
) -> Option<&'a EpistemeRegistryEntry> {
    entries.iter().find(|entry| entry.id == registry_id)
}

pub(crate) fn candidate_inspection_report_batch(
    report: &CandidateReadModelDuckDbInspectionReport,
) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("schema_version", DataType::Utf8, false),
        Field::new("execution_engine", DataType::Utf8, false),
        Field::new("registration_strategy", DataType::Utf8, false),
        Field::new("candidate_object_count", DataType::UInt64, false),
        Field::new("candidate_relation_count", DataType::UInt64, false),
        Field::new("candidate_evidence_count", DataType::UInt64, false),
        Field::new("review_status_violation_count", DataType::UInt64, false),
        Field::new("promotion_status_violation_count", DataType::UInt64, false),
        Field::new("ontology_truth_violation_count", DataType::UInt64, false),
        Field::new(
            "raw_to_rdf_promotion_violation_count",
            DataType::UInt64,
            false,
        ),
        Field::new("missing_relation_endpoint_count", DataType::UInt64, false),
        Field::new("inspection_passed", DataType::Boolean, false),
    ]));
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(vec![report.schema_version])),
        Arc::new(StringArray::from(vec![report.execution_engine])),
        Arc::new(StringArray::from(vec![report.registration_strategy])),
        Arc::new(UInt64Array::from(vec![
            report.candidate_object_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.candidate_relation_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.candidate_evidence_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.review_status_violation_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.promotion_status_violation_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.ontology_truth_violation_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.raw_to_rdf_promotion_violation_count as u64,
        ])),
        Arc::new(UInt64Array::from(vec![
            report.missing_relation_endpoint_count as u64,
        ])),
        Arc::new(BooleanArray::from(vec![report.inspection_passed])),
    ];
    RecordBatch::try_new(schema, columns)
        .map_err(|error| format!("failed to build candidate inspection Arrow batch: {error}"))
}
