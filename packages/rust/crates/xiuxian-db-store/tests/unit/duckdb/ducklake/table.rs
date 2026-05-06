use std::sync::Arc;

use super::{
    DuckLakeRecordBatchAppender, DuckLakeTableRef, append_ducklake_record_batches, must_err,
    must_ok,
};

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

#[test]
fn ducklake_record_batch_appender_reuses_open_appender_until_flush() {
    let connection = must_ok(
        ::duckdb::Connection::open_in_memory(),
        "open in-memory DuckDB for reusable DuckLake appender test",
    );
    must_ok(
        connection.execute_batch("CREATE TABLE events (event_type VARCHAR);"),
        "create appender target table",
    );

    let table = DuckLakeTableRef::main_schema("memory", "events");
    let mut appender = must_ok(
        DuckLakeRecordBatchAppender::open(&connection, &table),
        "open reusable DuckLake appender",
    );
    let first_rows = must_ok(
        appender.append_batch(event_type_batch(["tool.call"])),
        "append first batch through reusable appender",
    );
    let second_rows = must_ok(
        appender.append_batches([
            event_type_batch(["llm.call"]),
            event_type_batch(["bpmn.step", "tool.call"]),
        ]),
        "append additional batches through reusable appender",
    );

    assert_eq!(first_rows, 1);
    assert_eq!(second_rows, 3);
    assert_eq!(appender.rows_appended(), 4);

    must_ok(appender.flush(), "flush reusable DuckLake appender");
    let event_count: i64 = must_ok(
        connection.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0)),
        "query appended rows",
    );
    assert_eq!(event_count, 4);
}

fn event_type_batch<const N: usize>(
    event_types: [&str; N],
) -> ::duckdb::arrow::record_batch::RecordBatch {
    let schema = Arc::new(::duckdb::arrow::datatypes::Schema::new(vec![
        ::duckdb::arrow::datatypes::Field::new(
            "event_type",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
    ]));
    must_ok(
        ::duckdb::arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![Arc::new(::duckdb::arrow::array::StringArray::from(
                event_types.to_vec(),
            ))],
        ),
        "build reusable appender event batch",
    )
}
