use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt32Array, UInt64Array,
};
#[cfg(feature = "julia")]
use arrow::compute::concat_batches;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use xiuxian_wendao::duckdb::{
    DatasetOntologyArrowIpcSourceTableSpec, DatasetOntologyDuckDbMaterializer,
    DatasetOntologyRuntimeMaterialization, DatasetOntologyRuntimeMaterializationReport,
    DatasetOntologyRuntimeMaterializationRequest, DatasetOntologyRuntimeReadModelTable,
    DatasetOntologyWendaoGraphProofEvidence, DuckDbDatabasePath, SearchDuckDbExecutionConfig,
    SearchDuckDbRuntimeConfig, encode_dataset_ontology_materialization_app_metadata,
};
#[cfg(feature = "julia")]
use xiuxian_wendao::duckdb::{
    build_dataset_ontology_wendaograph_quality_request_batches,
    summarize_dataset_ontology_wendaograph_quality_response,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
};
use xiuxian_wendao_runtime::config::{
    DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS, DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE,
    DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW, DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
    DEFAULT_SEARCH_DUCKDB_THREADS,
};
use xiuxian_wendao_server::transport::{
    DatasetOntologyFlightManifest, DatasetOntologyMaterializeFlightRouteProvider,
    DatasetOntologyMaterializeFlightRouteResponse,
};
use xiuxian_wendao_sql::dataset_ontology::{
    DatasetOntologyMappingSql, DatasetOntologyMaterializedTableCount, DatasetOntologyValidationRule,
};

use crate::studio::GatewayState;
#[cfg(feature = "julia")]
use crate::studio::load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml;

const HEALTHCARE_CONTRACT_ID: &str = "healthcare.synthetic_care_delivery.contract.v1";
const HEALTHCARE_MAPPING_ID: &str = "healthcare.synthetic_care_delivery.v1";
const DATASET_ONTOLOGY_PAYLOAD_CACHE_RELATIVE_DIR: &[&str] =
    &[".cache", "ontology", "dataset-payloads"];
#[cfg(feature = "julia")]
const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV: &str =
    "WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL";
#[cfg(feature = "julia")]
const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_TIMEOUT_SECONDS_ENV: &str =
    "WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_TIMEOUT_SECONDS";

/// Studio-owned dataset ontology materialization provider for the Gateway
/// Flight service.
#[derive(Clone)]
pub(crate) struct StudioDatasetOntologyMaterializeFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioDatasetOntologyMaterializeFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioDatasetOntologyMaterializeFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioDatasetOntologyMaterializeFlightRouteProvider")
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
impl DatasetOntologyMaterializeFlightRouteProvider
    for StudioDatasetOntologyMaterializeFlightRouteProvider
{
    async fn dataset_ontology_materialize_batch(
        &self,
        manifest: &DatasetOntologyFlightManifest,
    ) -> Result<DatasetOntologyMaterializeFlightRouteResponse, String> {
        let source_table_specs = self.resolve_source_table_specs(manifest)?;
        let mapping_sql = load_mapping_sql(self.state.studio.project_root.as_path(), manifest)?;
        let request = DatasetOntologyRuntimeMaterializationRequest::from_arrow_ipc_streams(
            manifest.contract_id.as_str(),
            manifest.mapping_id.as_str(),
            &source_table_specs,
            mapping_sql,
        )?;
        let materializer = DatasetOntologyDuckDbMaterializer::from_runtime(
            dataset_ontology_duckdb_runtime(self.state.studio.project_root.as_path())?,
        )?;
        let materialization = materializer
            .materialize_with_read_model_batches(request)
            .await?;
        validate_report_against_manifest(&materialization.report, manifest)?;
        if !materialization.report.passed() {
            return Err(format!(
                "dataset ontology materialization failed validation for contract `{}` mapping `{}`: {}",
                manifest.contract_id,
                manifest.mapping_id,
                serde_json::to_string(&materialization.report.materialization.validation_failures)
                    .unwrap_or_else(|_| "validation failures could not be encoded".to_string())
            ));
        }
        let batches = materialization_result_batches(&materialization)?;
        #[cfg(feature = "julia")]
        let wendaograph_proof = dataset_ontology_wendaograph_proof_evidence(
            self.state.studio.config_root.as_path(),
            &materialization,
        )
        .await?;
        #[cfg(not(feature = "julia"))]
        let wendaograph_proof = dataset_ontology_wendaograph_proof_evidence(
            self.state.studio.config_root.as_path(),
            &materialization,
        );
        let app_metadata =
            dataset_ontology_app_metadata(&materialization.report, wendaograph_proof.as_ref())?;
        Ok(
            DatasetOntologyMaterializeFlightRouteResponse::from_batches(batches)
                .with_app_metadata(app_metadata),
        )
    }
}

impl StudioDatasetOntologyMaterializeFlightRouteProvider {
    fn resolve_source_table_specs(
        &self,
        manifest: &DatasetOntologyFlightManifest,
    ) -> Result<Vec<DatasetOntologyArrowIpcSourceTableSpec>, String> {
        let payload_root = dataset_ontology_payload_root(
            self.state.studio.project_root.as_path(),
            manifest.contract_id.as_str(),
            manifest.mapping_id.as_str(),
        )?;
        manifest
            .tables
            .iter()
            .map(|table| {
                let payload_id =
                    safe_cache_component(table.payload_id.as_str(), "dataset ontology payload id")?;
                let path = payload_root.join(format!("{payload_id}.arrow"));
                if !path.is_file() {
                    return Err(format!(
                        "dataset ontology Arrow IPC payload `{}` for table `{}` is missing at `{}`",
                        table.payload_id,
                        table.table_name,
                        path.display()
                    ));
                }
                validate_optional_content_sha256(
                    path.as_path(),
                    table.content_sha256.as_deref(),
                    table.payload_id.as_str(),
                )?;
                DatasetOntologyArrowIpcSourceTableSpec::new(table.table_name.as_str(), path)
            })
            .collect()
    }
}

fn dataset_ontology_payload_root(
    project_root: &Path,
    contract_id: &str,
    mapping_id: &str,
) -> Result<PathBuf, String> {
    let mut root = project_root.to_path_buf();
    for component in DATASET_ONTOLOGY_PAYLOAD_CACHE_RELATIVE_DIR {
        root.push(component);
    }
    root.push(safe_cache_component(
        contract_id,
        "dataset ontology contract id",
    )?);
    root.push(safe_cache_component(
        mapping_id,
        "dataset ontology mapping id",
    )?);
    Ok(root)
}

fn safe_cache_component<'a>(value: &'a str, label: &str) -> Result<&'a str, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} must not be blank"));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(format!("{label} `{value}` is not a safe cache component"));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        return Err(format!("{label} `{value}` is not a safe cache component"));
    }
    Ok(trimmed)
}

fn validate_optional_content_sha256(
    path: &Path,
    expected: Option<&str>,
    payload_id: &str,
) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read dataset ontology Arrow IPC payload `{payload_id}` for fingerprint validation: {error}"
        )
    })?;
    let digest = Sha256::digest(bytes);
    let mut actual = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut actual, "{byte:02x}")
            .map_err(|error| format!("failed to encode SHA-256 digest: {error}"))?;
    }
    if actual != expected {
        return Err(format!(
            "dataset ontology Arrow IPC payload `{payload_id}` content fingerprint mismatch: expected `{expected}`, got `{actual}`"
        ));
    }
    Ok(())
}

fn load_mapping_sql(
    project_root: &Path,
    manifest: &DatasetOntologyFlightManifest,
) -> Result<DatasetOntologyMappingSql, String> {
    if manifest.contract_id != HEALTHCARE_CONTRACT_ID
        || manifest.mapping_id != HEALTHCARE_MAPPING_ID
    {
        return Err(format!(
            "dataset ontology mapping `{}` for contract `{}` is not registered in Studio",
            manifest.mapping_id, manifest.contract_id
        ));
    }
    let ontology_root = project_root.join("wendao-episteme").join("ontology");
    Ok(DatasetOntologyMappingSql {
        object_observations: read_ontology_sql(
            ontology_root.as_path(),
            "30_Healthcare/mappings/sql/01_object_observations.sql",
        )?
        .into(),
        link_observations: read_ontology_sql(
            ontology_root.as_path(),
            "30_Healthcare/mappings/sql/02_link_observations.sql",
        )?
        .into(),
        evidence: read_ontology_sql(
            ontology_root.as_path(),
            "30_Healthcare/mappings/sql/03_evidence.sql",
        )?
        .into(),
        semantic_objects: read_ontology_sql(
            ontology_root.as_path(),
            "30_Healthcare/mappings/sql/04_semantic_objects.sql",
        )?
        .into(),
        semantic_relations: read_ontology_sql(
            ontology_root.as_path(),
            "30_Healthcare/mappings/sql/05_semantic_relations.sql",
        )?
        .into(),
        semantic_projection_state: read_ontology_sql(
            ontology_root.as_path(),
            "30_Healthcare/mappings/sql/06_semantic_projection_state.sql",
        )?
        .into(),
        validation_rules: vec![DatasetOntologyValidationRule::new(
            "HEALTHCARE_ENCOUNTER_MISSING_CONTEXT",
            read_ontology_sql(
                ontology_root.as_path(),
                "30_Healthcare/rules/01_encounter_must_link_patient_provider.sql",
            )?,
        )],
    })
}

fn read_ontology_sql(ontology_root: &Path, relative_path: &str) -> Result<String, String> {
    fs::read_to_string(ontology_root.join(relative_path)).map_err(|error| {
        format!(
            "failed to read dataset ontology SQL source contract `{relative_path}` from `{}`: {error}",
            ontology_root.display()
        )
    })
}

fn dataset_ontology_duckdb_runtime(
    project_root: &Path,
) -> Result<SearchDuckDbRuntimeConfig, String> {
    let temp_directory = project_root
        .join(".cache")
        .join("duckdb")
        .join("dataset-ontology");
    fs::create_dir_all(&temp_directory).map_err(|error| {
        format!(
            "failed to create dataset ontology DuckDB temp directory `{}`: {error}",
            temp_directory.display()
        )
    })?;
    Ok(SearchDuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory,
        threads: DEFAULT_SEARCH_DUCKDB_THREADS,
        execution: SearchDuckDbExecutionConfig {
            preserve_insertion_order: DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
            parquet_metadata_cache: DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE,
            prefer_virtual_arrow: DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS,
    })
}

fn validate_report_against_manifest(
    report: &DatasetOntologyRuntimeMaterializationReport,
    manifest: &DatasetOntologyFlightManifest,
) -> Result<(), String> {
    for table in &manifest.tables {
        let Some(expected_row_count) = table.row_count else {
            continue;
        };
        let actual_row_count = row_count_for_table(
            &report.materialization.source_tables,
            table.table_name.as_str(),
        )
        .ok_or_else(|| {
            format!(
                "dataset ontology materialization report omitted source table `{}`",
                table.table_name
            )
        })?;
        if actual_row_count != expected_row_count {
            return Err(format!(
                "dataset ontology source table `{}` row count mismatch: expected `{expected_row_count}`, got `{actual_row_count}`",
                table.table_name
            ));
        }
    }
    Ok(())
}

fn row_count_for_table(
    counts: &[DatasetOntologyMaterializedTableCount],
    table_name: &str,
) -> Option<u64> {
    counts
        .iter()
        .find(|count| count.table_name == table_name)
        .map(|count| count.row_count as u64)
}

fn dataset_ontology_app_metadata(
    report: &DatasetOntologyRuntimeMaterializationReport,
    wendaograph_proof: Option<&DatasetOntologyWendaoGraphProofEvidence>,
) -> Result<Vec<u8>, String> {
    encode_dataset_ontology_materialization_app_metadata(report, wendaograph_proof)
}

#[cfg(feature = "julia")]
async fn dataset_ontology_wendaograph_proof_evidence(
    config_root: &Path,
    materialization: &DatasetOntologyRuntimeMaterialization,
) -> Result<Option<DatasetOntologyWendaoGraphProofEvidence>, String> {
    let config =
        load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml(config_root);
    let base_url = config
        .as_ref()
        .map(|entry| entry.base_url.clone())
        .or_else(|| optional_env(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV));
    let Some(base_url) = base_url else {
        return Ok(None);
    };
    let binding = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url,
            health_route: None,
            timeout_secs: config.as_ref().and_then(|entry| entry.timeout_seconds).or(
                optional_u64_env(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_TIMEOUT_SECONDS_ENV)?,
            ),
            max_in_flight_requests: config
                .as_ref()
                .and_then(|entry| entry.max_in_flight_requests)
                .or(Some(1)),
        },
    )?;
    let batches = build_dataset_ontology_wendaograph_quality_request_batches(materialization)?;
    let Some(roundtrip) =
        roundtrip_wendaograph_ontology_read_model_quality_with_binding(&binding, &batches)
            .await
            .map_err(|error| {
                format!("dataset ontology WendaoGraph quality proof roundtrip failed: {error:?}")
            })?
    else {
        return Ok(None);
    };
    let response = proof_response_batch(&roundtrip.response_batches)?;
    summarize_dataset_ontology_wendaograph_quality_response(&response).map(Some)
}

#[cfg(not(feature = "julia"))]
fn dataset_ontology_wendaograph_proof_evidence(
    _project_root: &Path,
    _materialization: &DatasetOntologyRuntimeMaterialization,
) -> Option<DatasetOntologyWendaoGraphProofEvidence> {
    None
}

#[cfg(feature = "julia")]
fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(feature = "julia")]
fn optional_u64_env(name: &str) -> Result<Option<u64>, String> {
    optional_env(name)
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|error| format!("invalid `{name}` value `{value}`: {error}"))
        })
        .transpose()
}

#[cfg(feature = "julia")]
fn proof_response_batch(batches: &[RecordBatch]) -> Result<RecordBatch, String> {
    let Some(first) = batches.first() else {
        return Err("dataset ontology WendaoGraph proof response returned no batches".to_string());
    };
    if batches.len() == 1 {
        return Ok(first.clone());
    }
    concat_batches(&first.schema(), batches).map_err(|error| {
        format!("dataset ontology WendaoGraph proof response batches did not concatenate: {error}")
    })
}

fn materialization_result_batches(
    materialization: &DatasetOntologyRuntimeMaterialization,
) -> Result<Vec<RecordBatch>, String> {
    let mut batches = vec![materialization_report_batch(&materialization.report)?];
    for table in &materialization.read_model_tables {
        batches.push(read_model_table_batch(&materialization.report, table)?);
    }
    Ok(batches)
}

fn materialization_report_batch(
    report: &DatasetOntologyRuntimeMaterializationReport,
) -> Result<RecordBatch, String> {
    let report_json = serde_json::to_string(report)
        .map_err(|error| format!("failed to encode dataset ontology report JSON: {error}"))?;
    let payload_json_values = vec![report_json];
    envelope_batch(
        report,
        "materialization_report",
        "materialization_report",
        &payload_json_values,
    )
}

fn read_model_table_batch(
    report: &DatasetOntologyRuntimeMaterializationReport,
    table: &DatasetOntologyRuntimeReadModelTable,
) -> Result<RecordBatch, String> {
    let mut row_json_values = Vec::new();
    for batch in table.batches() {
        for row_index in 0..batch.num_rows() {
            row_json_values.push(record_batch_row_json(batch, row_index)?);
        }
    }
    envelope_batch(
        report,
        "semantic_read_model",
        table.table_name(),
        &row_json_values,
    )
}

fn envelope_batch(
    report: &DatasetOntologyRuntimeMaterializationReport,
    record_kind: &str,
    table_name: &str,
    payload_json_values: &[String],
) -> Result<RecordBatch, String> {
    let row_count = payload_json_values.len();
    let schema = dataset_ontology_envelope_schema();
    let contract_ids = vec![report.contract_id.as_str(); row_count];
    let mapping_ids = vec![report.mapping_id.as_str(); row_count];
    let record_kinds = vec![record_kind; row_count];
    let table_names = vec![table_name; row_count];
    let passed = vec![report.passed(); row_count];
    let execution_engines = vec![report.materialization.execution_engine.as_str(); row_count];
    let source_table_counts = vec![report.source_table_count as u64; row_count];
    let observation_table_counts =
        vec![report.materialization.observation_tables.len() as u64; row_count];
    let semantic_read_model_table_counts =
        vec![report.materialization.semantic_read_model_tables.len() as u64; row_count];
    let validation_failure_counts =
        vec![report.materialization.validation_failures.len() as u64; row_count];
    let row_indices = (0..row_count as u64).collect::<Vec<_>>();
    let payload_json_refs = payload_json_values
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(contract_ids)),
        Arc::new(StringArray::from(mapping_ids)),
        Arc::new(StringArray::from(record_kinds)),
        Arc::new(StringArray::from(table_names)),
        Arc::new(UInt64Array::from(row_indices)),
        Arc::new(BooleanArray::from(passed)),
        Arc::new(StringArray::from(execution_engines)),
        Arc::new(UInt64Array::from(source_table_counts)),
        Arc::new(UInt64Array::from(observation_table_counts)),
        Arc::new(UInt64Array::from(semantic_read_model_table_counts)),
        Arc::new(UInt64Array::from(validation_failure_counts)),
        Arc::new(StringArray::from(payload_json_refs)),
    ];
    RecordBatch::try_new(schema, columns)
        .map_err(|error| format!("failed to build dataset ontology envelope batch: {error}"))
}

fn dataset_ontology_envelope_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("contractId", DataType::Utf8, false),
        Field::new("mappingId", DataType::Utf8, false),
        Field::new("recordKind", DataType::Utf8, false),
        Field::new("tableName", DataType::Utf8, false),
        Field::new("rowIndex", DataType::UInt64, false),
        Field::new("passed", DataType::Boolean, false),
        Field::new("executionEngine", DataType::Utf8, false),
        Field::new("sourceTableCount", DataType::UInt64, false),
        Field::new("observationTableCount", DataType::UInt64, false),
        Field::new("semanticReadModelTableCount", DataType::UInt64, false),
        Field::new("validationFailureCount", DataType::UInt64, false),
        Field::new("payloadJson", DataType::Utf8, false),
    ]))
}

fn record_batch_row_json(batch: &RecordBatch, row_index: usize) -> Result<String, String> {
    let mut object = serde_json::Map::new();
    for (column_index, field) in batch.schema().fields().iter().enumerate() {
        object.insert(
            field.name().clone(),
            arrow_value_json(
                batch.column(column_index).as_ref(),
                field.data_type(),
                row_index,
            )?,
        );
    }
    serde_json::to_string(&serde_json::Value::Object(object))
        .map_err(|error| format!("failed to encode dataset ontology read-model row JSON: {error}"))
}

fn arrow_value_json(
    array: &dyn Array,
    data_type: &DataType,
    row_index: usize,
) -> Result<serde_json::Value, String> {
    if array.is_null(row_index) {
        return Ok(serde_json::Value::Null);
    }
    match data_type {
        DataType::Utf8 => Ok(serde_json::Value::String(
            downcast_value::<StringArray>(array, data_type)?
                .value(row_index)
                .to_string(),
        )),
        DataType::LargeUtf8 => Ok(serde_json::Value::String(
            downcast_value::<LargeStringArray>(array, data_type)?
                .value(row_index)
                .to_string(),
        )),
        DataType::Boolean => Ok(serde_json::Value::Bool(
            downcast_value::<BooleanArray>(array, data_type)?.value(row_index),
        )),
        DataType::Int32 => Ok(serde_json::Value::Number(serde_json::Number::from(
            downcast_value::<Int32Array>(array, data_type)?.value(row_index),
        ))),
        DataType::Int64 => Ok(serde_json::Value::Number(serde_json::Number::from(
            downcast_value::<Int64Array>(array, data_type)?.value(row_index),
        ))),
        DataType::UInt32 => Ok(serde_json::Value::Number(serde_json::Number::from(
            downcast_value::<UInt32Array>(array, data_type)?.value(row_index),
        ))),
        DataType::UInt64 => Ok(serde_json::Value::Number(serde_json::Number::from(
            downcast_value::<UInt64Array>(array, data_type)?.value(row_index),
        ))),
        DataType::Float32 => float_json(
            f64::from(downcast_value::<Float32Array>(array, data_type)?.value(row_index)),
            data_type,
        ),
        DataType::Float64 => float_json(
            downcast_value::<Float64Array>(array, data_type)?.value(row_index),
            data_type,
        ),
        other => Err(format!(
            "dataset ontology read-model row JSON does not support Arrow type `{other:?}`"
        )),
    }
}

fn float_json(value: f64, data_type: &DataType) -> Result<serde_json::Value, String> {
    serde_json::Number::from_f64(value)
        .map(serde_json::Value::Number)
        .ok_or_else(|| {
            format!("dataset ontology read-model row JSON cannot encode non-finite `{data_type:?}`")
        })
}

fn downcast_value<'a, T: 'static>(
    array: &'a dyn Array,
    data_type: &DataType,
) -> Result<&'a T, String> {
    array.as_any().downcast_ref::<T>().ok_or_else(|| {
        format!("dataset ontology Arrow array did not match expected type `{data_type:?}`")
    })
}
