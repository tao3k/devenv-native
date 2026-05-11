use serde_json::json;
use xiuxian_wendao_core::{SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload};

#[test]
fn sql_query_payload_round_trips_through_json() {
    let payload = SqlQueryPayload {
        metadata: SqlQueryMetadata {
            catalog_table_name: "catalog".to_string(),
            column_catalog_table_name: "columns".to_string(),
            view_source_catalog_table_name: "views".to_string(),
            supports_information_schema: true,
            registered_tables: vec!["markdown".to_string()],
            registered_table_count: 1,
            registered_view_count: 0,
            registered_column_count: 4,
            registered_view_source_count: 0,
            result_batch_count: 1,
            result_row_count: 1,
            registered_input_bytes: Some(10.into()),
            result_bytes: Some(20.into()),
            local_relation_materialization_state: Some("materialized".to_string().into()),
            local_temp_storage_peak_bytes: Some(30.into()),
            local_relation_engine: Some("duckdb".to_string()),
            duckdb_registration_strategy: Some("view".to_string()),
            registered_input_batch_count: Some(1),
            registered_input_row_count: Some(1),
            registration_time_ms: Some(2.into()),
            local_query_execution_time_ms: Some(3.into()),
        },
        batches: vec![SqlBatchPayload {
            row_count: 1,
            columns: vec![SqlColumnPayload {
                name: "path".to_string(),
                data_type: "Utf8".to_string().into(),
                nullable: false,
            }],
            rows: vec![serde_json::Map::from_iter([(
                "path".to_string(),
                json!("docs/a.md"),
            )])],
        }],
    };

    let encoded = match serde_json::to_value(&payload) {
        Ok(value) => value,
        Err(error) => panic!("payload should serialize: {error}"),
    };
    assert_eq!(encoded["metadata"]["catalogTableName"], "catalog");
    assert_eq!(encoded["batches"][0]["columns"][0]["dataType"], "Utf8");

    let decoded = match serde_json::from_value::<SqlQueryPayload>(encoded) {
        Ok(value) => value,
        Err(error) => panic!("payload should deserialize: {error}"),
    };
    assert_eq!(decoded, payload);
}
