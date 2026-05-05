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
    let runtime = DuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.path().join("duckdb-tmp"),
        threads: 1,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: false,
            prefer_virtual_arrow: true,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: 10,
    };
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
            r#"
            CREATE TABLE wendao_lake.events (
              tenant_id VARCHAR,
              case_id VARCHAR,
              event_type VARCHAR,
              payload VARCHAR,
              created_at VARCHAR
            );
            "#,
        ),
        "create DuckLake event table",
    );
    let schema = std::sync::Arc::new(::duckdb::arrow::datatypes::Schema::new(vec![
        ::duckdb::arrow::datatypes::Field::new(
            "tenant_id",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
        ::duckdb::arrow::datatypes::Field::new(
            "case_id",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
        ::duckdb::arrow::datatypes::Field::new(
            "event_type",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
        ::duckdb::arrow::datatypes::Field::new(
            "payload",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
        ::duckdb::arrow::datatypes::Field::new(
            "created_at",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
    ]));
    let batch = must_ok(
        ::duckdb::arrow::record_batch::RecordBatch::try_new(
            schema,
            vec![
                std::sync::Arc::new(::duckdb::arrow::array::StringArray::from(vec!["tenant-a"]))
                    as ::duckdb::arrow::array::ArrayRef,
                std::sync::Arc::new(::duckdb::arrow::array::StringArray::from(vec!["case-1"])),
                std::sync::Arc::new(::duckdb::arrow::array::StringArray::from(vec!["tool.call"])),
                std::sync::Arc::new(::duckdb::arrow::array::StringArray::from(vec![
                    r#"{"tool":"probe"}"#,
                ])),
                std::sync::Arc::new(::duckdb::arrow::array::StringArray::from(vec![
                    "2026-05-04T00:00:00Z",
                ])),
            ],
        ),
        "build Arrow event batch",
    );
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
