//! DuckDB-backed dataset-to-ontology runtime handoff.

use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(feature = "julia")]
use std::collections::BTreeSet;

#[cfg(feature = "julia")]
use arrow::array::StringArray;
#[cfg(feature = "julia")]
use arrow::compute::concat_batches;
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    WendaoGraphOntologyExtensionProofRequestBatches,
    WendaoGraphOntologyReadModelQualityRequestBatches,
};
use xiuxian_wendao_runtime::config::SearchDuckDbRuntimeConfig;
use xiuxian_wendao_sql::dataset_ontology::{
    DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME, DatasetOntologyMappingSql,
    DatasetOntologyMaterializationReport, DatasetOntologySourceTable,
    materialize_dataset_ontology_with_engine,
};

use super::{DuckDbLocalRelationEngine, LocalRelationEngine};

/// Named Arrow IPC source table supplied by a source-contract handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetOntologyArrowIpcSourceTableSpec {
    table_name: String,
    path: PathBuf,
}

impl DatasetOntologyArrowIpcSourceTableSpec {
    /// Build one named Arrow IPC source table spec.
    ///
    /// # Errors
    ///
    /// Returns an error when the table name is empty.
    pub fn new(table_name: impl Into<String>, path: impl Into<PathBuf>) -> Result<Self, String> {
        let table_name = table_name.into();
        if table_name.trim().is_empty() {
            return Err("dataset ontology Arrow IPC table name must not be empty".to_string());
        }
        Ok(Self {
            table_name,
            path: path.into(),
        })
    }

    /// Stable source table name to register in `DuckDB`.
    #[must_use]
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Arrow IPC stream file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Runtime input for one dataset-to-ontology materialization request.
pub struct DatasetOntologyRuntimeMaterializationRequest {
    contract_id: String,
    mapping_id: String,
    source_tables: Vec<DatasetOntologySourceTable>,
    mapping_sql: DatasetOntologyMappingSql,
}

impl DatasetOntologyRuntimeMaterializationRequest {
    /// Build one runtime materialization request.
    ///
    /// # Errors
    ///
    /// Returns an error when the contract id or mapping id is empty.
    pub fn new(
        contract_id: impl Into<String>,
        mapping_id: impl Into<String>,
        source_tables: Vec<DatasetOntologySourceTable>,
        mapping_sql: DatasetOntologyMappingSql,
    ) -> Result<Self, String> {
        let contract_id = contract_id.into();
        if contract_id.trim().is_empty() {
            return Err("dataset ontology contract id must not be empty".to_string());
        }
        let mapping_id = mapping_id.into();
        if mapping_id.trim().is_empty() {
            return Err("dataset ontology mapping id must not be empty".to_string());
        }
        Ok(Self {
            contract_id,
            mapping_id,
            source_tables,
            mapping_sql,
        })
    }

    /// Build one runtime request from named Arrow IPC stream files.
    ///
    /// # Errors
    ///
    /// Returns an error when identifiers are empty, an IPC file cannot be
    /// opened, the IPC stream cannot be decoded, or the decoded table cannot be
    /// accepted as a source table.
    pub fn from_arrow_ipc_streams(
        contract_id: impl Into<String>,
        mapping_id: impl Into<String>,
        source_table_specs: &[DatasetOntologyArrowIpcSourceTableSpec],
        mapping_sql: DatasetOntologyMappingSql,
    ) -> Result<Self, String> {
        let source_tables = source_table_specs
            .iter()
            .map(read_dataset_ontology_arrow_ipc_source_table)
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(contract_id, mapping_id, source_tables, mapping_sql)
    }

    /// Contract identifier from the accepted ontology source contract.
    #[must_use]
    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    /// Mapping identifier from the accepted ontology source contract.
    #[must_use]
    pub fn mapping_id(&self) -> &str {
        &self.mapping_id
    }

    /// Number of raw source tables supplied by the caller.
    #[must_use]
    pub fn source_table_count(&self) -> usize {
        self.source_tables.len()
    }
}

/// Read one Arrow IPC stream file into a dataset ontology source table.
///
/// # Errors
///
/// Returns an error when the file cannot be opened, the IPC stream cannot be
/// decoded, or the decoded batches do not match the stream schema.
pub fn read_dataset_ontology_arrow_ipc_source_table(
    spec: &DatasetOntologyArrowIpcSourceTableSpec,
) -> Result<DatasetOntologySourceTable, String> {
    let file = File::open(spec.path()).map_err(|error| {
        format!(
            "failed to open dataset ontology Arrow IPC source table `{}` at `{}`: {error}",
            spec.table_name(),
            spec.path().display()
        )
    })?;
    let mut reader = StreamReader::try_new(file, None).map_err(|error| {
        format!(
            "failed to decode dataset ontology Arrow IPC source table `{}` at `{}`: {error}",
            spec.table_name(),
            spec.path().display()
        )
    })?;
    let schema = reader.schema();
    let batches = reader
        .by_ref()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to read dataset ontology Arrow IPC source table `{}` at `{}`: {error}",
                spec.table_name(),
                spec.path().display()
            )
        })?;
    DatasetOntologySourceTable::new(spec.table_name().to_string(), schema, batches)
}

/// Runtime report for one DuckDB-backed dataset-to-ontology materialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologyRuntimeMaterializationReport {
    /// Contract identifier from the accepted ontology source contract.
    pub contract_id: String,
    /// Mapping identifier from the accepted ontology source contract.
    pub mapping_id: String,
    /// Number of raw source tables supplied by the caller.
    pub source_table_count: usize,
    /// DuckDB-backed materialization details and validation failures.
    pub materialization: DatasetOntologyMaterializationReport,
}

impl DatasetOntologyRuntimeMaterializationReport {
    /// Whether the underlying materialization report passed all validation
    /// rules.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.materialization.passed()
    }
}

/// Materialized read-model table emitted by the runtime materializer.
pub struct DatasetOntologyRuntimeReadModelTable {
    table_name: String,
    batches: Vec<RecordBatch>,
}

impl DatasetOntologyRuntimeReadModelTable {
    fn new(table_name: impl Into<String>, batches: Vec<RecordBatch>) -> Result<Self, String> {
        let table_name = table_name.into();
        if table_name.trim().is_empty() {
            return Err("dataset ontology read-model table name must not be empty".to_string());
        }
        Ok(Self {
            table_name,
            batches,
        })
    }

    /// Stable read-model table name.
    #[must_use]
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Materialized Arrow batches for the read-model table.
    #[must_use]
    pub fn batches(&self) -> &[RecordBatch] {
        &self.batches
    }

    /// Total row count across all batches.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.batches.iter().map(RecordBatch::num_rows).sum()
    }
}

/// Full runtime materialization result, including report metadata and compiled
/// read-model table batches.
pub struct DatasetOntologyRuntimeMaterialization {
    /// Compact materialization report.
    pub report: DatasetOntologyRuntimeMaterializationReport,
    /// Materialized semantic read-model tables.
    pub read_model_tables: Vec<DatasetOntologyRuntimeReadModelTable>,
}

/// Build the `WendaoGraph` ontology quality request from Rust-owned dataset materialization.
///
/// # Errors
///
/// Returns an error when materialization validation failed, required read-model
/// tables are missing or empty, Arrow batches cannot be merged, or semantic
/// relation endpoints do not resolve to known semantic object ids.
#[cfg(feature = "julia")]
pub fn build_dataset_ontology_wendaograph_quality_request_batches(
    materialization: &DatasetOntologyRuntimeMaterialization,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    if !materialization.report.passed() {
        return Err(format!(
            "dataset ontology materialization failed validation: {:?}",
            materialization.report.materialization.validation_failures
        ));
    }

    let objects = runtime_read_model_batch(
        materialization,
        DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    )?;
    let relations = runtime_read_model_batch(
        materialization,
        DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
    )?;
    let projection_state = runtime_read_model_batch(
        materialization,
        DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    )?;

    validate_dataset_ontology_relation_endpoints(&objects, &relations)?;

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        objects,
        relations,
        projection_state,
    ))
}

/// Build the `WendaoGraph` ontology extension proof request from dataset facts
/// and caller-supplied compiled parent registry tables.
///
/// # Errors
///
/// Returns an error when the dataset read-model cannot be adapted to the
/// quality request contract.
#[cfg(feature = "julia")]
pub fn build_dataset_ontology_wendaograph_extension_proof_request_batches(
    materialization: &DatasetOntologyRuntimeMaterialization,
    parent_object_types: RecordBatch,
    parent_link_types: RecordBatch,
) -> Result<WendaoGraphOntologyExtensionProofRequestBatches, String> {
    let read_model = build_dataset_ontology_wendaograph_quality_request_batches(materialization)?;
    Ok(WendaoGraphOntologyExtensionProofRequestBatches::new(
        parent_object_types,
        parent_link_types,
        read_model,
    ))
}

/// DuckDB-backed dataset-to-ontology materializer owned by `xiuxian-wendao`.
pub struct DatasetOntologyDuckDbMaterializer {
    engine: DuckDbLocalRelationEngine,
}

impl DatasetOntologyDuckDbMaterializer {
    /// Open a materializer from merged Wendao `DuckDB` runtime settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured `DuckDB` runtime cannot be opened.
    pub fn configured() -> Result<Self, String> {
        DuckDbLocalRelationEngine::configured().map(Self::from_engine)
    }

    /// Open a materializer from one resolved `DuckDB` runtime config.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured `DuckDB` runtime cannot be opened.
    pub fn from_runtime(runtime: SearchDuckDbRuntimeConfig) -> Result<Self, String> {
        DuckDbLocalRelationEngine::from_runtime(runtime).map(Self::from_engine)
    }

    /// Wrap an existing `DuckDB` local relation engine.
    #[must_use]
    pub fn from_engine(engine: DuckDbLocalRelationEngine) -> Self {
        Self { engine }
    }

    /// Materialize one dataset-to-ontology request through `DuckDB`.
    ///
    /// # Errors
    ///
    /// Returns an error when source table registration, SELECT-only SQL
    /// admission, mapping execution, or validation execution fails.
    pub async fn materialize(
        &self,
        request: DatasetOntologyRuntimeMaterializationRequest,
    ) -> Result<DatasetOntologyRuntimeMaterializationReport, String> {
        let materialization = materialize_dataset_ontology_with_engine(
            &self.engine,
            &request.source_tables,
            &request.mapping_sql,
        )
        .await?;
        Ok(DatasetOntologyRuntimeMaterializationReport {
            contract_id: request.contract_id,
            mapping_id: request.mapping_id,
            source_table_count: request.source_tables.len(),
            materialization,
        })
    }

    /// Materialize one dataset-to-ontology request and return the compiled
    /// semantic read-model table batches.
    ///
    /// # Errors
    ///
    /// Returns an error when materialization fails or when any semantic
    /// read-model table cannot be queried after materialization.
    pub async fn materialize_with_read_model_batches(
        &self,
        request: DatasetOntologyRuntimeMaterializationRequest,
    ) -> Result<DatasetOntologyRuntimeMaterialization, String> {
        let report = self.materialize(request).await?;
        let read_model_tables = self.read_model_tables().await?;
        Ok(DatasetOntologyRuntimeMaterialization {
            report,
            read_model_tables,
        })
    }

    async fn read_model_tables(&self) -> Result<Vec<DatasetOntologyRuntimeReadModelTable>, String> {
        let mut tables = Vec::new();
        for table_name in [
            DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
            DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
            DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
        ] {
            let sql = format!("select * from {table_name}");
            let batches = self.engine.query_batches(&sql).await?;
            tables.push(DatasetOntologyRuntimeReadModelTable::new(
                table_name, batches,
            )?);
        }
        Ok(tables)
    }
}

#[cfg(feature = "julia")]
fn runtime_read_model_batch(
    materialization: &DatasetOntologyRuntimeMaterialization,
    table_name: &str,
) -> Result<RecordBatch, String> {
    let table = materialization
        .read_model_tables
        .iter()
        .find(|table| table.table_name() == table_name)
        .ok_or_else(|| format!("dataset ontology read-model table `{table_name}` is missing"))?;

    match table.batches() {
        [] => Err(format!(
            "dataset ontology read-model table `{table_name}` has no Arrow batches"
        )),
        [batch] => Ok(batch.clone()),
        batches => concat_batches(&batches[0].schema(), batches.iter()).map_err(|error| {
            format!("merge dataset ontology read-model table `{table_name}` batches: {error}")
        }),
    }
}

#[cfg(feature = "julia")]
fn validate_dataset_ontology_relation_endpoints(
    objects: &RecordBatch,
    relations: &RecordBatch,
) -> Result<(), String> {
    let object_ids = string_column(objects, "id", DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME)?;
    let mut known_object_ids = BTreeSet::new();
    for row_index in 0..objects.num_rows() {
        known_object_ids.insert(object_ids.value(row_index).to_owned());
    }

    let relation_sources = string_column(
        relations,
        "source",
        DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
    )?;
    let relation_targets = string_column(
        relations,
        "target",
        DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
    )?;
    for row_index in 0..relations.num_rows() {
        let source = relation_sources.value(row_index);
        if !known_object_ids.contains(source) {
            return Err(format!(
                "dataset ontology relation row {row_index} references unknown source object `{source}`"
            ));
        }
        let target = relation_targets.value(row_index);
        if !known_object_ids.contains(target) {
            return Err(format!(
                "dataset ontology relation row {row_index} references unknown target object `{target}`"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "julia")]
fn string_column<'a>(
    batch: &'a RecordBatch,
    column_name: &str,
    table_name: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            format!(
                "dataset ontology read-model table `{table_name}` must contain string column `{column_name}`"
            )
        })
}
