//! DuckDB-backed dataset-to-ontology runtime handoff.

use std::fs::File;
use std::path::{Path, PathBuf};

use arrow::ipc::reader::StreamReader;
use serde::{Deserialize, Serialize};
use xiuxian_wendao_runtime::config::SearchDuckDbRuntimeConfig;
use xiuxian_wendao_sql::dataset_ontology::{
    DatasetOntologyMappingSql, DatasetOntologyMaterializationReport, DatasetOntologySourceTable,
    materialize_dataset_ontology_with_engine,
};

use super::DuckDbLocalRelationEngine;

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
}
