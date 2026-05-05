use super::{
    build_duckdb_parquet_view_sql, build_duckdb_virtual_view_sql, ensure_duckdb_identifier, must_ok,
};

#[test]
fn duckdb_sql_helpers_validate_and_escape_inputs() {
    assert!(ensure_duckdb_identifier("workflow_state", "table").is_ok());
    assert!(ensure_duckdb_identifier("9workflow", "table").is_err());

    let parquet_sql = must_ok(
        build_duckdb_parquet_view_sql("workflow_state", std::path::Path::new("data's.parquet")),
        "valid parquet view SQL",
    );
    assert!(parquet_sql.contains("read_parquet('data''s.parquet')"));

    let virtual_sql = must_ok(
        build_duckdb_virtual_view_sql("workflow_state", "ns'1", "arrow_relation"),
        "valid virtual view SQL",
    );
    assert!(virtual_sql.contains("arrow_relation('ns''1', 'workflow_state')"));
}
