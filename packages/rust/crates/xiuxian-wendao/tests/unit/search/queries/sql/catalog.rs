use xiuxian_wendao_runtime::transport::SqlFlightRouteProvider;

use crate::search::queries::sql::provider::StudioSqlFlightRouteProvider;
use crate::search::queries::sql::provider::metadata::StudioSqlFlightMetadata;
use crate::search::queries::sql::registration::{
    STUDIO_SQL_CATALOG_TABLE_NAME, STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
    STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME, build_columns_catalog_batch,
    build_tables_catalog_batch, build_view_sources_catalog_batch,
};
use crate::search::queries::sql::tests::fixtures::{
    bool_column_values, fixture_service, nullable_string_column_values, publish_reference_hits,
    sample_hit, string_column_values, u64_column_values,
};
use crate::search::{SearchCorpusKind, SearchPlaneService};
use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

#[test]
fn studio_sql_catalog_batches_use_wendao_table_metadata() {
    let tables_batch = build_tables_catalog_batch(&[])
        .unwrap_or_else(|error| panic!("build tables catalog batch: {error}"));
    let columns_batch = build_columns_catalog_batch(&[])
        .unwrap_or_else(|error| panic!("build columns catalog batch: {error}"));
    let view_sources_batch = build_view_sources_catalog_batch(&[])
        .unwrap_or_else(|error| panic!("build view sources catalog batch: {error}"));

    assert_schema_table_name(&tables_batch.schema(), STUDIO_SQL_CATALOG_TABLE_NAME);
    assert_schema_table_name(
        &columns_batch.schema(),
        STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
    );
    assert_schema_table_name(
        &view_sources_batch.schema(),
        STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME,
    );
}

#[tokio::test]
async fn studio_sql_flight_provider_exposes_registered_tables_catalog() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaService", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    let epoch = publish_reference_hits(&service, "fp-sql-catalog-1", &hits).await;
    let provider = StudioSqlFlightRouteProvider::new(service.clone());
    let engine_table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::ReferenceOccurrence,
        epoch,
    );
    let sql_table_name = SearchCorpusKind::ReferenceOccurrence.to_string();

    let response = provider
        .sql_query_batches(
            format!(
                "SELECT sql_table_name, engine_table_name, corpus, scope, sql_object_kind, source_count, repo_id FROM {STUDIO_SQL_CATALOG_TABLE_NAME} ORDER BY sql_table_name"
            )
            .as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("catalog query batches: {error}"));

    assert_eq!(response.batches.len(), 1);
    assert_eq!(response.batches[0].num_rows(), 4);
    assert_eq!(
        string_column_values(&response.batches[0], "sql_table_name"),
        vec![
            sql_table_name.clone(),
            STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME.to_string(),
            STUDIO_SQL_CATALOG_TABLE_NAME.to_string(),
            STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME.to_string(),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "engine_table_name"),
        vec![
            engine_table_name.clone(),
            STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME.to_string(),
            STUDIO_SQL_CATALOG_TABLE_NAME.to_string(),
            STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME.to_string(),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "corpus"),
        vec![
            "reference_occurrence".to_string(),
            "system".to_string(),
            "system".to_string(),
            "system".to_string(),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "scope"),
        vec![
            "local".to_string(),
            "system".to_string(),
            "system".to_string(),
            "system".to_string(),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "sql_object_kind"),
        vec![
            "table".to_string(),
            "system".to_string(),
            "system".to_string(),
            "system".to_string(),
        ]
    );
    assert_eq!(
        u64_column_values(&response.batches[0], "source_count"),
        vec![0, 0, 0, 0]
    );
    assert_eq!(
        nullable_string_column_values(&response.batches[0], "repo_id"),
        vec![None, None, None, None]
    );

    let app_metadata: StudioSqlFlightMetadata =
        serde_json::from_slice(response.app_metadata.as_slice())
            .unwrap_or_else(|error| panic!("decode app metadata: {error}"));
    assert_eq!(
        app_metadata.catalog_table_name,
        STUDIO_SQL_CATALOG_TABLE_NAME
    );
    assert_eq!(
        app_metadata.column_catalog_table_name,
        STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME
    );
    assert_eq!(
        app_metadata.view_source_catalog_table_name,
        STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME
    );
}

#[tokio::test]
async fn studio_sql_flight_provider_exposes_registered_columns_catalog() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaService", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    let epoch = publish_reference_hits(&service, "fp-sql-catalog-columns-1", &hits).await;
    let provider = StudioSqlFlightRouteProvider::new(service.clone());
    let _engine_table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::ReferenceOccurrence,
        epoch,
    );
    let sql_table_name = SearchCorpusKind::ReferenceOccurrence.to_string();

    let response = provider
        .sql_query_batches(
            format!(
                "SELECT column_name, source_column_name, data_type, is_nullable, ordinal_position, sql_object_kind, column_origin_kind FROM {STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME} WHERE sql_table_name = '{sql_table_name}' ORDER BY ordinal_position"
            )
            .as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("column catalog query batches: {error}"));

    assert_eq!(response.batches.len(), 1);
    assert_eq!(response.batches[0].num_rows(), 8);
    assert_eq!(
        string_column_values(&response.batches[0], "column_name"),
        vec![
            "id".to_string(),
            "name".to_string(),
            "name_folded".to_string(),
            "path".to_string(),
            "line".to_string(),
            "column".to_string(),
            "line_text".to_string(),
            "hit_json".to_string(),
        ]
    );
    assert_eq!(
        nullable_string_column_values(&response.batches[0], "source_column_name"),
        vec![
            Some("id".to_string()),
            Some("name".to_string()),
            Some("name_folded".to_string()),
            Some("path".to_string()),
            Some("line".to_string()),
            Some("column".to_string()),
            Some("line_text".to_string()),
            Some("hit_json".to_string()),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "data_type"),
        vec![
            "Utf8".to_string(),
            "Utf8".to_string(),
            "Utf8".to_string(),
            "Utf8".to_string(),
            "UInt64".to_string(),
            "UInt64".to_string(),
            "Utf8".to_string(),
            "Utf8".to_string(),
        ]
    );
    assert_eq!(
        bool_column_values(&response.batches[0], "is_nullable"),
        vec![false, false, false, false, false, false, false, false]
    );
    assert_eq!(
        u64_column_values(&response.batches[0], "ordinal_position"),
        vec![1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "sql_object_kind"),
        vec![
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "column_origin_kind"),
        vec![
            "stored".to_string(),
            "stored".to_string(),
            "stored".to_string(),
            "stored".to_string(),
            "stored".to_string(),
            "stored".to_string(),
            "stored".to_string(),
            "stored".to_string(),
        ]
    );
}

fn assert_schema_table_name(schema: &arrow::datatypes::Schema, expected: &str) {
    assert_eq!(
        schema
            .metadata()
            .get(WENDAO_TABLE_METADATA_KEY)
            .map(String::as_str),
        Some(expected)
    );
}
