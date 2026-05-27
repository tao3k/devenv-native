use std::path::Path;
use std::sync::Arc;

use arrow::array::{Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use chrono::{TimeZone, Utc};
use serde_json::json;
use xiuxian_db_store::{
    WENDAO_TABLE_METADATA_KEY,
    duckdb::{DuckLakeCatalog, DuckLakeDataPath},
};

use super::{TestResult, in_memory_search_duckdb_runtime};
use crate::duckdb::{
    WENDAO_EVENT_APPEND_DEFAULT_BATCH_ROWS, WENDAO_EVENT_LAKE_DEFAULT_ALIAS,
    WENDAO_EVENT_LAKE_EVENTS_TABLE, WENDAO_EVENT_QUERY_DEFAULT_LIMIT, WENDAO_EVENT_QUERY_MAX_LIMIT,
    WendaoEventLake, WendaoEventLakeLocalConfig, WendaoEventQuery, WendaoEventRecord,
    build_wendao_event_lake_table_sql, open_search_duckdb_connection, validate_wendao_event_batch,
    wendao_event_record_batch, wendao_event_schema,
};

#[test]
fn wendao_event_lake_schema_sql_and_batch_contract_are_stable() -> TestResult {
    let sql = build_wendao_event_lake_table_sql(WENDAO_EVENT_LAKE_DEFAULT_ALIAS)
        .map_err(std::io::Error::other)?;
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"wendao_lake\".\"events\""));
    assert!(sql.contains("payload VARCHAR"));
    assert!(sql.contains("created_at TIMESTAMP"));

    let created_at = Utc
        .with_ymd_and_hms(2026, 5, 5, 8, 0, 0)
        .single()
        .ok_or_else(|| std::io::Error::other("valid UTC timestamp"))?;
    let batch = wendao_event_record_batch(&[WendaoEventRecord::new(
        "tenant-a",
        "case-1",
        "tool.call",
        &json!({"tool": "probe"}),
        created_at,
    )])
    .map_err(std::io::Error::other)?;

    assert_eq!(batch.schema().as_ref(), wendao_event_schema().as_ref());
    assert_eq!(
        batch
            .schema()
            .metadata()
            .get(WENDAO_TABLE_METADATA_KEY)
            .map(String::as_str),
        Some("wendao_event_lake_events")
    );
    assert_eq!(batch.num_rows(), 1);
    let payloads = batch
        .column(3)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| std::io::Error::other("payload column is string array"))?;
    assert_eq!(payloads.value(0), r#"{"tool":"probe"}"#);
    validate_wendao_event_batch(&batch).map_err(std::io::Error::other)?;

    let empty_batch = wendao_event_record_batch(&[]).map_err(std::io::Error::other)?;
    assert_eq!(
        empty_batch.schema().as_ref(),
        wendao_event_schema().as_ref()
    );
    assert_eq!(empty_batch.num_rows(), 0);

    let invalid_schema = Arc::new(Schema::new(vec![Field::new(
        "tenant_id",
        DataType::Utf8,
        false,
    )]));
    let invalid_batch = RecordBatch::try_new(
        invalid_schema,
        vec![Arc::new(StringArray::from(vec!["tenant-a"]))],
    )?;
    assert!(validate_wendao_event_batch(&invalid_batch).is_err());

    Ok(())
}

#[test]
fn wendao_event_lake_handle_validates_alias_and_table_ref() -> TestResult {
    let lake = WendaoEventLake::attached(WENDAO_EVENT_LAKE_DEFAULT_ALIAS)
        .map_err(std::io::Error::other)?;
    assert_eq!(lake.catalog_alias(), WENDAO_EVENT_LAKE_DEFAULT_ALIAS);

    let default_lake = WendaoEventLake::default_alias();
    assert_eq!(
        default_lake.catalog_alias(),
        WENDAO_EVENT_LAKE_DEFAULT_ALIAS
    );

    let table_ref = lake.events_table_ref();
    assert_eq!(table_ref.catalog_alias, WENDAO_EVENT_LAKE_DEFAULT_ALIAS);
    assert_eq!(table_ref.schema_name, "main");
    assert_eq!(table_ref.table_name, WENDAO_EVENT_LAKE_EVENTS_TABLE);

    assert!(WendaoEventLake::attached("9lake").is_err());

    Ok(())
}

#[test]
fn wendao_event_lake_local_config_derives_paths_and_attach_config() -> TestResult {
    let temp = tempfile::tempdir()?;
    let config =
        WendaoEventLakeLocalConfig::from_data_home(temp.path()).map_err(std::io::Error::other)?;

    assert_eq!(config.catalog_alias(), WENDAO_EVENT_LAKE_DEFAULT_ALIAS);
    assert_eq!(
        config.event_lake_root(),
        temp.path().join("wendao").join("event_lake")
    );
    assert_eq!(
        config.metadata_path(),
        temp.path()
            .join("wendao")
            .join("event_lake")
            .join("metadata")
            .join("wendao.ducklake")
    );
    assert_eq!(
        config.data_path(),
        temp.path().join("wendao").join("event_lake").join("data")
    );

    let attach_config = config.ducklake_attach_config();
    assert_eq!(attach_config.alias, WENDAO_EVENT_LAKE_DEFAULT_ALIAS);
    assert_eq!(
        attach_config.catalog,
        DuckLakeCatalog::local_metadata_file(config.metadata_path())
    );
    assert_eq!(
        attach_config.data_path,
        DuckLakeDataPath::local(config.data_path())
    );

    assert!(WendaoEventLakeLocalConfig::from_data_home_with_alias("9lake", temp.path()).is_err());

    Ok(())
}

#[test]
fn wendao_event_query_contract_validates_filters_and_limits() {
    let default_query = WendaoEventQuery::new();
    assert_eq!(default_query.limit, WENDAO_EVENT_QUERY_DEFAULT_LIMIT);
    assert!(default_query.validate().is_ok());

    let case_query = WendaoEventQuery::for_case("tenant-a", "case-1")
        .with_event_type("tool.call")
        .with_limit(25);
    assert_eq!(case_query.tenant_id.as_deref(), Some("tenant-a"));
    assert_eq!(case_query.case_id.as_deref(), Some("case-1"));
    assert_eq!(case_query.event_type.as_deref(), Some("tool.call"));
    assert_eq!(case_query.limit, 25);
    assert!(case_query.validate().is_ok());

    assert!(WendaoEventQuery::new().with_limit(0).validate().is_err());
    assert!(
        WendaoEventQuery::new()
            .with_limit(WENDAO_EVENT_QUERY_MAX_LIMIT + 1)
            .validate()
            .is_err()
    );
    assert!(
        WendaoEventQuery::new()
            .with_tenant_id("   ")
            .validate()
            .is_err()
    );
}

#[test]
#[ignore = "requires downloading/loading DuckDB's ducklake extension"]
fn wendao_event_lake_live_smoke_appends_arrow_events_and_queries_counts() -> TestResult {
    let temp = tempfile::tempdir()?;
    let (connection, lake) = open_live_event_lake(temp.path())?;
    let (events, batch) = live_smoke_events()?;

    append_live_smoke_events(&connection, &lake, &events, batch)?;
    assert_live_smoke_counts(&connection, &lake)?;
    assert_live_smoke_case_query(&connection, &lake)?;
    assert_live_smoke_tool_query(&connection, &lake)?;
    assert_live_smoke_json_payload_count(&connection)?;

    Ok(())
}

fn open_live_event_lake(
    temp_root: &Path,
) -> Result<(duckdb::Connection, WendaoEventLake), Box<dyn std::error::Error>> {
    let runtime = in_memory_search_duckdb_runtime(temp_root);
    let connection = open_search_duckdb_connection(&runtime).map_err(std::io::Error::other)?;
    let config =
        WendaoEventLakeLocalConfig::from_data_home(temp_root).map_err(std::io::Error::other)?;
    let lake = config.attach(&connection).map_err(std::io::Error::other)?;
    Ok((connection, lake))
}

fn live_smoke_events() -> Result<(Vec<WendaoEventRecord>, RecordBatch), Box<dyn std::error::Error>>
{
    let first_at = live_smoke_timestamp(0, "first")?;
    let second_at = live_smoke_timestamp(1, "second")?;
    let third_at = live_smoke_timestamp(2, "third")?;
    let events = vec![
        WendaoEventRecord::new(
            "tenant-a",
            "case-1",
            "tool.call",
            &json!({"tool": "probe"}),
            first_at,
        ),
        WendaoEventRecord::new(
            "tenant-a",
            "case-1",
            "llm.call",
            &json!({"model": "local"}),
            second_at,
        ),
        WendaoEventRecord::new(
            "tenant-a",
            "case-2",
            "tool.call",
            &json!({"tool": "search"}),
            second_at,
        ),
    ];
    let batch_events = vec![WendaoEventRecord::new(
        "tenant-a",
        "case-1",
        "bpmn.step",
        &json!({"node": "approve"}),
        third_at,
    )];
    let batch = wendao_event_record_batch(&batch_events).map_err(std::io::Error::other)?;
    Ok((events, batch))
}

fn live_smoke_timestamp(
    minute_offset: u32,
    label: &str,
) -> Result<chrono::DateTime<Utc>, Box<dyn std::error::Error>> {
    Utc.with_ymd_and_hms(2026, 5, 5, 8, minute_offset, 0)
        .single()
        .ok_or_else(|| std::io::Error::other(format!("valid {label} UTC timestamp")).into())
}

fn append_live_smoke_events(
    connection: &duckdb::Connection,
    lake: &WendaoEventLake,
    events: &[WendaoEventRecord],
    batch: RecordBatch,
) -> TestResult {
    let mut appender = lake
        .open_appender(connection)
        .map_err(std::io::Error::other)?;
    assert!(
        appender
            .append_events_chunked(events, 0)
            .map_err(std::io::Error::other)
            .is_err()
    );
    let event_row_count = appender
        .append_events_chunked(events, 2)
        .map_err(std::io::Error::other)?;
    let batch_row_count = appender
        .append_batches(std::iter::once(batch))
        .map_err(std::io::Error::other)?;
    appender.flush().map_err(std::io::Error::other)?;

    assert_eq!(event_row_count, 3);
    assert_eq!(batch_row_count, 1);
    assert_eq!(appender.rows_appended(), 4);
    assert_eq!(WENDAO_EVENT_APPEND_DEFAULT_BATCH_ROWS, 4_096);
    Ok(())
}

fn assert_live_smoke_counts(connection: &duckdb::Connection, lake: &WendaoEventLake) -> TestResult {
    let counts = lake
        .event_type_counts(connection)
        .map_err(std::io::Error::other)?;
    assert_eq!(counts.len(), 3);
    assert_eq!(counts[0].event_type, "bpmn.step");
    assert_eq!(counts[0].count, 1);
    assert_eq!(counts[1].event_type, "llm.call");
    assert_eq!(counts[1].count, 1);
    assert_eq!(counts[2].event_type, "tool.call");
    assert_eq!(counts[2].count, 2);
    Ok(())
}

fn assert_live_smoke_case_query(
    connection: &duckdb::Connection,
    lake: &WendaoEventLake,
) -> TestResult {
    let queried_events = lake
        .query_events(
            connection,
            &WendaoEventQuery::for_case("tenant-a", "case-1").with_limit(10),
        )
        .map_err(std::io::Error::other)?;
    assert_eq!(queried_events.len(), 3);
    assert_eq!(queried_events[0].event_type, "tool.call");
    assert_eq!(queried_events[1].event_type, "llm.call");
    assert_eq!(queried_events[2].event_type, "bpmn.step");
    assert_eq!(queried_events[2].payload_json(), r#"{"node":"approve"}"#);
    assert_eq!(
        queried_events[2]
            .payload_value()
            .map_err(std::io::Error::other)?,
        json!({"node": "approve"})
    );
    Ok(())
}

fn assert_live_smoke_tool_query(
    connection: &duckdb::Connection,
    lake: &WendaoEventLake,
) -> TestResult {
    let queried_tool_events = lake
        .query_events(
            connection,
            &WendaoEventQuery::for_case("tenant-a", "case-1")
                .with_event_type("tool.call")
                .with_limit(10),
        )
        .map_err(std::io::Error::other)?;
    assert_eq!(queried_tool_events.len(), 1);
    assert_eq!(queried_tool_events[0].case_id, "case-1");
    assert_eq!(queried_tool_events[0].event_type, "tool.call");
    Ok(())
}

fn assert_live_smoke_json_payload_count(connection: &duckdb::Connection) -> TestResult {
    let json_payload_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM wendao_lake.events WHERE json_valid(payload)",
            [],
            |row| row.get(0),
        )
        .map_err(std::io::Error::other)?;
    assert_eq!(json_payload_count, 4);

    Ok(())
}
