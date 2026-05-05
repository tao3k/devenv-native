use super::{DuckLakeTableRef, append_ducklake_record_batches, must_err, must_ok};

#[test]
fn ducklake_table_refs_validate_catalog_schema_and_table_names() {
    let events = DuckLakeTableRef::main_schema("wendao_lake", "events");
    assert_eq!(events.catalog_alias, "wendao_lake");
    assert_eq!(events.schema_name, "main");
    assert_eq!(events.table_name, "events");

    let workflow_events = DuckLakeTableRef::new("wendao_lake", "workflow", "events");
    assert_eq!(workflow_events.catalog_alias, "wendao_lake");
    assert_eq!(workflow_events.schema_name, "workflow");
    assert_eq!(workflow_events.table_name, "events");

    let connection = must_ok(
        ::duckdb::Connection::open_in_memory(),
        "open in-memory DuckDB for invalid DuckLake table validation",
    );
    let invalid_table = DuckLakeTableRef::main_schema("wendao_lake", "9events");
    let error = must_err(
        append_ducklake_record_batches(&connection, &invalid_table, Vec::new()),
        "invalid DuckLake table should fail before opening an appender",
    );
    assert!(error.contains("DuckLake table"));
}
