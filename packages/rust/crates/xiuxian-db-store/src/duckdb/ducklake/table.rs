//! Fully qualified `DuckLake` table references for appender helpers.

use serde::{Deserialize, Serialize};

use crate::duckdb::ensure_duckdb_identifier;

/// Fully qualified table reference inside an attached DuckLake catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuckLakeTableRef {
    /// Attached DuckLake catalog alias.
    pub catalog_alias: String,
    /// DuckDB schema name inside the DuckLake catalog.
    pub schema_name: String,
    /// Table name inside the schema.
    pub table_name: String,
}

impl DuckLakeTableRef {
    /// Build a DuckLake table reference in the default `main` schema.
    #[must_use]
    pub fn main_schema(catalog_alias: impl Into<String>, table_name: impl Into<String>) -> Self {
        Self {
            catalog_alias: catalog_alias.into(),
            schema_name: "main".to_string(),
            table_name: table_name.into(),
        }
    }

    /// Build a DuckLake table reference with an explicit schema.
    #[must_use]
    pub fn new(
        catalog_alias: impl Into<String>,
        schema_name: impl Into<String>,
        table_name: impl Into<String>,
    ) -> Self {
        Self {
            catalog_alias: catalog_alias.into(),
            schema_name: schema_name.into(),
            table_name: table_name.into(),
        }
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        ensure_duckdb_identifier(&self.catalog_alias, "DuckLake catalog")?;
        ensure_duckdb_identifier(&self.schema_name, "DuckLake schema")?;
        ensure_duckdb_identifier(&self.table_name, "DuckLake table")?;
        Ok(())
    }
}
