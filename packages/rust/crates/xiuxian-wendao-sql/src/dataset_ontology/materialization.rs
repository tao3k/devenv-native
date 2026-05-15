//! Engine-neutral dataset-to-ontology materialization over local relations.

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

use crate::{LocalRelationEngine, LocalRelationRegistrationHint};

use super::sql::validate_dataset_ontology_select_only_sql;

/// Canonical object observation table name.
pub const DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME: &str = "ontology_object_observation";
/// Canonical link observation table name.
pub const DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME: &str = "ontology_link_observation";
/// Canonical evidence table name.
pub const DATASET_ONTOLOGY_EVIDENCE_TABLE_NAME: &str = "ontology_evidence";
/// Compatibility entity table name used by ontology validation SQL.
pub const DATASET_ONTOLOGY_ENTITY_TABLE_NAME: &str = "ontology_entity";
/// Compatibility relation table name used by ontology validation SQL.
pub const DATASET_ONTOLOGY_RELATION_TABLE_NAME: &str = "ontology_relation";
/// Semantic object read-model table name.
pub const DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME: &str = "semantic_objects";
/// Semantic relation read-model table name.
pub const DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME: &str = "semantic_relations";
/// Semantic projection-state read-model table name.
pub const DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME: &str = "semantic_projection_state";

const ENTITY_COMPATIBILITY_SQL: &str =
    "select object_id as entity_id, rdf_class as class_iri from ontology_object_observation";
const RELATION_COMPATIBILITY_SQL: &str = "select source_object_id as source_id, target_object_id as target_id, rdf_property as predicate from ontology_link_observation";

/// Raw source table batches supplied by the runtime handoff.
#[derive(Clone)]
pub struct DatasetOntologySourceTable {
    name: String,
    schema: SchemaRef,
    batches: Vec<RecordBatch>,
}

impl DatasetOntologySourceTable {
    /// Create one raw source table from Arrow batches.
    ///
    /// # Errors
    ///
    /// Returns an error when the table name is empty or when any batch schema
    /// differs from the declared schema.
    pub fn new(
        name: impl Into<String>,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<Self, String> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err("dataset ontology source table name must not be empty".to_string());
        }
        for batch in &batches {
            if batch.schema().as_ref() != schema.as_ref() {
                return Err(format!(
                    "dataset ontology source table `{name}` received a mismatched batch schema"
                ));
            }
        }
        Ok(Self {
            name,
            schema,
            batches,
        })
    }

    /// Stable table name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    fn row_count(&self) -> usize {
        record_batch_row_count(&self.batches)
    }
}

/// SELECT-only mapping SQL needed for one dataset-to-ontology contract.
#[derive(Debug, Clone)]
pub struct DatasetOntologyMappingSql {
    /// Object observation projection SQL.
    pub object_observations: DatasetOntologySelectSql,
    /// Link observation projection SQL.
    pub link_observations: DatasetOntologySelectSql,
    /// Evidence projection SQL.
    pub evidence: DatasetOntologySelectSql,
    /// Semantic object read-model projection SQL.
    pub semantic_objects: DatasetOntologySelectSql,
    /// Semantic relation read-model projection SQL.
    pub semantic_relations: DatasetOntologySelectSql,
    /// Semantic projection-state projection SQL.
    pub semantic_projection_state: DatasetOntologySelectSql,
    /// Domain validation rules to execute after compatibility views exist.
    pub validation_rules: Vec<DatasetOntologyValidationRule>,
}

/// Typed SELECT SQL text for dataset-to-ontology projection contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetOntologySelectSql(String);

impl DatasetOntologySelectSql {
    /// Create one SELECT SQL wrapper.
    #[must_use]
    pub fn new(sql: impl Into<String>) -> Self {
        Self(sql.into())
    }

    /// Borrow the wrapped SQL text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for DatasetOntologySelectSql {
    fn from(sql: String) -> Self {
        Self::new(sql)
    }
}

impl From<&str> for DatasetOntologySelectSql {
    fn from(sql: &str) -> Self {
        Self::new(sql)
    }
}

/// One named validation query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologyValidationRule {
    /// Stable rule identifier.
    pub rule_id: String,
    /// SELECT-only validation SQL. Returned rows are validation failures.
    pub sql: String,
}

impl DatasetOntologyValidationRule {
    /// Create one named validation rule.
    #[must_use]
    pub fn new(rule_id: impl Into<String>, sql: impl Into<String>) -> Self {
        Self {
            rule_id: rule_id.into(),
            sql: sql.into(),
        }
    }
}

/// Row-count evidence for one materialized table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologyMaterializedTableCount {
    /// Table name.
    pub table_name: String,
    /// Number of rows observed.
    pub row_count: usize,
}

/// Validation rule result that returned at least one failure row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologyValidationFailure {
    /// Stable rule identifier.
    pub rule_id: String,
    /// Number of failure rows returned by the rule query.
    pub row_count: usize,
}

/// Runtime report for one dataset-to-ontology materialization pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOntologyMaterializationReport {
    /// Engine label that executed the materialization pass.
    pub execution_engine: String,
    /// Raw source table row counts.
    pub source_tables: Vec<DatasetOntologyMaterializedTableCount>,
    /// Observation table row counts.
    pub observation_tables: Vec<DatasetOntologyMaterializedTableCount>,
    /// Semantic read-model table row counts.
    pub semantic_read_model_tables: Vec<DatasetOntologyMaterializedTableCount>,
    /// Validation rules that returned failure rows.
    pub validation_failures: Vec<DatasetOntologyValidationFailure>,
}

impl DatasetOntologyMaterializationReport {
    /// Whether all validation rules returned zero failure rows.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.validation_failures.is_empty()
    }

    /// Return one materialized row count by table name.
    #[must_use]
    pub fn row_count_for(&self, table_name: &str) -> Option<usize> {
        self.source_tables
            .iter()
            .chain(self.observation_tables.iter())
            .chain(self.semantic_read_model_tables.iter())
            .find(|table| table.table_name == table_name)
            .map(|table| table.row_count)
    }
}

/// Materialize one dataset-to-ontology mapping over a local relation engine.
///
/// # Errors
///
/// Returns an error when raw table registration fails, when any mapping SQL is
/// not SELECT-only, when query execution fails, or when a derived query returns
/// no schema-bearing batches.
pub async fn materialize_dataset_ontology_with_engine(
    query_engine: &impl LocalRelationEngine,
    source_tables: &[DatasetOntologySourceTable],
    mapping_sql: &DatasetOntologyMappingSql,
) -> Result<DatasetOntologyMaterializationReport, String> {
    let mut source_table_counts = Vec::with_capacity(source_tables.len());
    for source_table in source_tables {
        query_engine.register_record_batches_with_hint(
            source_table.name(),
            source_table.schema.clone(),
            source_table.batches.clone(),
            LocalRelationRegistrationHint::RepeatedUse,
        )?;
        source_table_counts.push(table_count(source_table.name(), source_table.row_count()));
    }

    let object_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME,
        mapping_sql.object_observations.as_str(),
    )
    .await?;
    let link_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME,
        mapping_sql.link_observations.as_str(),
    )
    .await?;
    let evidence_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_EVIDENCE_TABLE_NAME,
        mapping_sql.evidence.as_str(),
    )
    .await?;
    let entity_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_ENTITY_TABLE_NAME,
        ENTITY_COMPATIBILITY_SQL,
    )
    .await?;
    let relation_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_RELATION_TABLE_NAME,
        RELATION_COMPATIBILITY_SQL,
    )
    .await?;

    let semantic_objects_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
        mapping_sql.semantic_objects.as_str(),
    )
    .await?;
    let semantic_relations_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
        mapping_sql.semantic_relations.as_str(),
    )
    .await?;
    let semantic_projection_state_count = query_and_register(
        query_engine,
        DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
        mapping_sql.semantic_projection_state.as_str(),
    )
    .await?;

    let mut validation_failures = Vec::new();
    for rule in &mapping_sql.validation_rules {
        validate_dataset_ontology_select_only_sql(&rule.sql)?;
        let batches = query_engine.query_batches(&rule.sql).await?;
        let row_count = record_batch_row_count(&batches);
        if row_count > 0 {
            validation_failures.push(DatasetOntologyValidationFailure {
                rule_id: rule.rule_id.clone(),
                row_count,
            });
        }
    }

    Ok(DatasetOntologyMaterializationReport {
        execution_engine: query_engine.kind().as_str().to_string(),
        source_tables: source_table_counts,
        observation_tables: vec![
            table_count(DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME, object_count),
            table_count(DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME, link_count),
            table_count(DATASET_ONTOLOGY_EVIDENCE_TABLE_NAME, evidence_count),
            table_count(DATASET_ONTOLOGY_ENTITY_TABLE_NAME, entity_count),
            table_count(DATASET_ONTOLOGY_RELATION_TABLE_NAME, relation_count),
        ],
        semantic_read_model_tables: vec![
            table_count(
                DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
                semantic_objects_count,
            ),
            table_count(
                DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
                semantic_relations_count,
            ),
            table_count(
                DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
                semantic_projection_state_count,
            ),
        ],
        validation_failures,
    })
}

async fn query_and_register(
    query_engine: &impl LocalRelationEngine,
    table_name: &str,
    sql: &str,
) -> Result<usize, String> {
    validate_dataset_ontology_select_only_sql(sql)?;
    let batches = query_engine.query_batches(sql).await?;
    let schema = batches
        .first()
        .map(RecordBatch::schema)
        .ok_or_else(|| format!("dataset ontology query for `{table_name}` returned no batches"))?;
    let row_count = record_batch_row_count(&batches);
    query_engine.register_record_batches_with_hint(
        table_name,
        schema,
        batches,
        LocalRelationRegistrationHint::RepeatedUse,
    )?;
    Ok(row_count)
}

fn table_count(table_name: &str, row_count: usize) -> DatasetOntologyMaterializedTableCount {
    DatasetOntologyMaterializedTableCount {
        table_name: table_name.to_string(),
        row_count,
    }
}

fn record_batch_row_count(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}
