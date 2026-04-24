use std::path::Path;

/// Builds SQL that drops any temp view or table registered with a `DuckDB` name.
///
/// The caller must pass a valid `DuckDB` identifier. Use
/// [`ensure_duckdb_identifier`] when accepting user or external input.
#[must_use]
pub fn build_drop_duckdb_registered_relation_sql(table_name: &str) -> String {
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    format!("DROP VIEW IF EXISTS {quoted_table_name};\nDROP TABLE IF EXISTS {quoted_table_name};")
}

/// Builds SQL that exposes one registered Arrow table function as a temp view.
///
/// # Errors
///
/// Returns an error when the function or table identifiers are not supported by
/// the bounded `DuckDB` helper contract.
pub fn build_duckdb_virtual_view_sql(
    table_name: &str,
    namespace: &str,
    function_name: &str,
) -> Result<String, String> {
    ensure_duckdb_identifier(table_name, "table")?;
    ensure_duckdb_identifier(function_name, "function")?;
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    let escaped_namespace = namespace.replace('\'', "''");
    let escaped_table_name = table_name.replace('\'', "''");
    Ok(format!(
        "{drop_relation_sql}\nCREATE TEMP VIEW {quoted_table_name} AS SELECT * FROM {function_name}('{escaped_namespace}', '{escaped_table_name}');",
        drop_relation_sql = build_drop_duckdb_registered_relation_sql(table_name)
    ))
}

/// Builds SQL that exposes a file or directory of Parquet data as a temp view.
///
/// # Errors
///
/// Returns an error when the table identifier is not supported by the bounded
/// `DuckDB` helper contract.
pub fn build_duckdb_parquet_view_sql(
    table_name: &str,
    table_path: &Path,
) -> Result<String, String> {
    ensure_duckdb_identifier(table_name, "table")?;
    let quoted_table_name = quoted_duckdb_identifier(table_name);
    let read_path = if table_path.is_dir() {
        table_path.join("*.parquet")
    } else {
        table_path.to_path_buf()
    };
    let escaped_path = read_path.to_string_lossy().replace('\'', "''");
    Ok(format!(
        "{drop_sql}\nCREATE TEMP VIEW {quoted_table_name} AS SELECT * FROM read_parquet('{escaped_path}');",
        drop_sql = build_drop_duckdb_registered_relation_sql(table_name),
    ))
}

/// Validates one identifier for the bounded `DuckDB` helper contract.
///
/// # Errors
///
/// Returns an error when the identifier is blank, starts with an unsupported
/// character, or contains unsupported characters.
pub fn ensure_duckdb_identifier(identifier: &str, label: &str) -> Result<(), String> {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return Err(format!(
            "duckdb local relation {label} identifiers cannot be blank"
        ));
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(format!(
            "duckdb local relation {label} identifiers must start with an ASCII letter or underscore: `{identifier}`"
        ));
    }
    if !chars.all(|character| character.is_ascii_alphanumeric() || character == '_') {
        return Err(format!(
            "duckdb local relation {label} identifiers must only use ASCII letters, digits, or underscores: `{identifier}`"
        ));
    }
    Ok(())
}

/// Quotes one identifier that has already passed [`ensure_duckdb_identifier`].
#[must_use]
pub fn quoted_duckdb_identifier(identifier: &str) -> String {
    format!("\"{identifier}\"")
}
