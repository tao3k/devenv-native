//! `DuckDB` implementation of the local relation engine contract.

use std::collections::BTreeMap;
use std::sync::Mutex;

use arrow::datatypes::{DataType, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use duckdb::Connection;

use super::types::{
    LocalRelationEngine, LocalRelationEngineKind, LocalRelationMaterializationState,
    LocalRelationRegistrationHint,
};

const DUCKDB_REGISTRATION_STRATEGY: &str = "duckdb_materialized_arrow_staging";

/// Default bounded local relation engine backed by an in-memory `DuckDB` connection.
pub struct DuckDbLocalRelationEngine {
    connection: Mutex<Connection>,
    registrations: Mutex<BTreeMap<String, LocalRelationMaterializationState>>,
}

impl DuckDbLocalRelationEngine {
    /// Create a fresh in-memory `DuckDB` local relation engine.
    ///
    /// # Errors
    ///
    /// Returns an error when the in-memory `DuckDB` connection cannot be opened.
    pub fn new_in_memory() -> Result<Self, String> {
        let connection = Connection::open_in_memory()
            .map_err(|error| format!("failed to open in-memory DuckDB connection: {error}"))?;
        Ok(Self {
            connection: Mutex::new(connection),
            registrations: Mutex::new(BTreeMap::new()),
        })
    }
}

#[async_trait]
impl LocalRelationEngine for DuckDbLocalRelationEngine {
    fn kind(&self) -> LocalRelationEngineKind {
        LocalRelationEngineKind::DuckDb
    }

    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<(), String> {
        self.register_record_batches_with_hint(
            table_name,
            schema,
            batches,
            LocalRelationRegistrationHint::Default,
        )
    }

    fn register_record_batches_with_hint(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
        hint: LocalRelationRegistrationHint,
    ) -> Result<(), String> {
        let _ = hint;
        validate_record_batches(table_name, &schema, &batches)?;
        let create_sql = create_table_sql(table_name, &schema)?;
        {
            let connection = self.connection.lock().map_err(|_| {
                format!("failed to lock DuckDB connection while registering `{table_name}`")
            })?;
            connection
                .execute_batch(&format!(
                    "DROP TABLE IF EXISTS {};{}",
                    quote_identifier(table_name),
                    create_sql
                ))
                .map_err(|error| {
                    format!("failed to create DuckDB table `{table_name}`: {error}")
                })?;
            if !batches.is_empty() {
                let mut appender = connection.appender(table_name).map_err(|error| {
                    format!("failed to open DuckDB appender for `{table_name}`: {error}")
                })?;
                for batch in batches {
                    appender.append_record_batch(batch).map_err(|error| {
                        format!(
                            "failed to append Arrow batch into DuckDB table `{table_name}`: {error}"
                        )
                    })?;
                }
                appender.flush().map_err(|error| {
                    format!("failed to flush DuckDB appender for `{table_name}`: {error}")
                })?;
            }
        }
        self.registrations
            .lock()
            .map_err(|_| format!("failed to lock DuckDB registration state for `{table_name}`"))?
            .insert(
                table_name.to_string(),
                LocalRelationMaterializationState::Materialized,
            );
        Ok(())
    }

    fn relation_registration_strategy(&self, table_name: &str) -> Option<&'static str> {
        self.registrations
            .lock()
            .ok()?
            .contains_key(table_name)
            .then_some(DUCKDB_REGISTRATION_STRATEGY)
    }

    fn relation_materialization_state(
        &self,
        table_name: &str,
    ) -> Option<LocalRelationMaterializationState> {
        self.registrations.lock().ok()?.get(table_name).copied()
    }

    async fn query_batches(&self, sql: &str) -> Result<Vec<RecordBatch>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "failed to lock DuckDB connection while querying".to_string())?;
        let mut statement = connection
            .prepare(sql)
            .map_err(|error| format!("failed to plan DuckDB SQL query: {error}"))?;
        statement
            .query_arrow([])
            .map_err(|error| format!("failed to execute DuckDB SQL query: {error}"))
            .map(Iterator::collect)
    }
}

fn validate_record_batches(
    table_name: &str,
    schema: &SchemaRef,
    batches: &[RecordBatch],
) -> Result<(), String> {
    for batch in batches {
        if batch.schema().as_ref() != schema.as_ref() {
            return Err(format!(
                "DuckDB table `{table_name}` received a mismatched Arrow batch schema"
            ));
        }
    }
    Ok(())
}

fn create_table_sql(table_name: &str, schema: &SchemaRef) -> Result<String, String> {
    let columns = schema
        .fields()
        .iter()
        .map(|field| {
            Ok(format!(
                "{} {}{}",
                quote_identifier(field.name()),
                duckdb_type(field.data_type())?,
                if field.is_nullable() { "" } else { " NOT NULL" }
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(format!(
        "CREATE TABLE {} ({});",
        quote_identifier(table_name),
        columns.join(", ")
    ))
}

fn duckdb_type(data_type: &DataType) -> Result<&'static str, String> {
    match data_type {
        DataType::Boolean => Ok("BOOLEAN"),
        DataType::Int8 => Ok("TINYINT"),
        DataType::Int16 => Ok("SMALLINT"),
        DataType::Int32 => Ok("INTEGER"),
        DataType::Int64 => Ok("BIGINT"),
        DataType::UInt8 => Ok("UTINYINT"),
        DataType::UInt16 => Ok("USMALLINT"),
        DataType::UInt32 => Ok("UINTEGER"),
        DataType::UInt64 => Ok("UBIGINT"),
        DataType::Float32 => Ok("REAL"),
        DataType::Float64 => Ok("DOUBLE"),
        DataType::Utf8 | DataType::LargeUtf8 => Ok("VARCHAR"),
        DataType::Binary | DataType::LargeBinary => Ok("BLOB"),
        DataType::Date32 => Ok("DATE"),
        DataType::Time64(TimeUnit::Microsecond) => Ok("TIME"),
        DataType::Timestamp(TimeUnit::Microsecond, _) => Ok("TIMESTAMP"),
        unsupported => Err(format!(
            "DuckDB local relation engine does not support Arrow type `{unsupported:?}`"
        )),
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(test)]
#[path = "../../tests/unit/local_relation/duckdb.rs"]
mod tests;
