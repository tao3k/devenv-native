use std::path::Path;
use std::sync::Arc;

use ::duckdb::arrow::{
    array::{ArrayRef, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use tempfile::tempdir;

use super::{
    DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, DuckLakeAttachConfig,
    DuckLakeTableRef, append_ducklake_record_batches, attach_ducklake, must_ok,
    open_duckdb_connection,
};

#[test]
#[ignore = "requires downloading/loading DuckDB's ducklake extension"]
fn ducklake_live_attach_smoke() {
    let root = must_ok(tempdir(), "create DuckLake live root");
    let runtime = live_duckdb_runtime(root.path());
    let connection = must_ok(
        open_duckdb_connection(&runtime),
        "open DuckDB for live DuckLake smoke",
    );
    let config = DuckLakeAttachConfig::local(
        "wendao_lake",
        root.path().join("metadata").join("local.ducklake"),
        root.path().join("data"),
    );

    must_ok(
        attach_ducklake(&connection, &config),
        "attach local DuckLake catalog",
    );
    must_ok(
        connection.execute_batch(
            r"
            CREATE TABLE wendao_lake.events (
              tenant_id VARCHAR,
              case_id VARCHAR,
              event_type VARCHAR,
              payload VARCHAR,
              created_at VARCHAR
            );
            ",
        ),
        "create DuckLake event table",
    );
    let batch = ducklake_event_batch();
    let appended_rows = must_ok(
        append_ducklake_record_batches(
            &connection,
            &DuckLakeTableRef::main_schema("wendao_lake", "events"),
            vec![batch],
        ),
        "append Arrow event batch into DuckLake",
    );
    assert_eq!(appended_rows, 1);

    let event_count: i64 = must_ok(
        connection.query_row(
            "SELECT COUNT(*) FROM wendao_lake.events WHERE event_type = 'tool.call'",
            [],
            |row| row.get(0),
        ),
        "query DuckLake event rows",
    );

    assert_eq!(event_count, 1);
}

fn live_duckdb_runtime(root: &Path) -> DuckDbRuntimeConfig {
    DuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.join("duckdb-tmp"),
        threads: 1,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: false,
            prefer_virtual_arrow: true,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: 10,
    }
}

fn ducklake_event_batch() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("tenant_id", DataType::Utf8, false),
        Field::new("case_id", DataType::Utf8, false),
        Field::new("event_type", DataType::Utf8, false),
        Field::new("payload", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
    ]));
    must_ok(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["tenant-a"])) as ArrayRef,
                Arc::new(StringArray::from(vec!["case-1"])),
                Arc::new(StringArray::from(vec!["tool.call"])),
                Arc::new(StringArray::from(vec![r#"{"tool":"probe"}"#])),
                Arc::new(StringArray::from(vec!["2026-05-04T00:00:00Z"])),
            ],
        ),
        "build Arrow event batch",
    )
}
