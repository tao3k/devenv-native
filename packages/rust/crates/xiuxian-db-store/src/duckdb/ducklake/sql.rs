//! `DuckDB` SQL builders for `DuckLake` extension bootstrap and attach.

use super::catalog::{DuckLakeAttachConfig, DuckLakeCatalog};
use crate::duckdb::{ensure_duckdb_identifier, quoted_duckdb_identifier};

/// Build SQL that installs and loads the extensions required by one `DuckLake`
/// catalog.
#[must_use]
pub fn build_ducklake_extension_bootstrap_sql(catalog: &DuckLakeCatalog) -> String {
    let mut statements = vec!["INSTALL ducklake", "LOAD ducklake"];
    if catalog.needs_postgres_extension() {
        statements.push("INSTALL postgres");
        statements.push("LOAD postgres");
    }
    format!("{};", statements.join(";\n"))
}

/// Build SQL that attaches one `DuckLake` catalog to the current `DuckDB`
/// connection.
///
/// # Errors
///
/// Returns an error when the alias, metadata catalog, or data path is outside
/// the generic helper contract.
pub fn build_ducklake_attach_sql(config: &DuckLakeAttachConfig) -> Result<String, String> {
    ensure_duckdb_identifier(&config.alias, "DuckLake catalog")?;
    let mut statements = Vec::new();
    if config.bootstrap_extensions {
        statements.push(build_ducklake_extension_bootstrap_sql(&config.catalog));
    }

    let attach_uri = escape_duckdb_string_literal(config.catalog.attach_uri()?.as_str());
    let data_path = escape_duckdb_string_literal(config.data_path_sql_value()?.as_str());
    let alias = quoted_duckdb_identifier(&config.alias);
    statements.push(format!(
        "ATTACH '{attach_uri}' AS {alias} (DATA_PATH '{data_path}');"
    ));
    Ok(statements.join("\n"))
}

/// Build SQL that selects one attached `DuckLake` catalog as the active database.
///
/// # Errors
///
/// Returns an error when the alias is outside the generic helper contract.
pub fn build_ducklake_use_sql(alias: &str) -> Result<String, String> {
    ensure_duckdb_identifier(alias, "DuckLake catalog")?;
    Ok(format!("USE {};", quoted_duckdb_identifier(alias)))
}

fn escape_duckdb_string_literal(value: &str) -> String {
    value.replace('\'', "''")
}
