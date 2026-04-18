use xiuxian_wendao_runtime::transport::SqlFlightRouteProvider;

use crate::search::queries::sql::provider::StudioSqlFlightRouteProvider;
use crate::search::queries::sql::registration::{
    STUDIO_SQL_CATALOG_TABLE_NAME, STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
};
use crate::search::queries::sql::tests::fixtures::{
    fixture_service, publish_reference_hits, sample_hit, string_column_values,
};
use crate::search::{SearchCorpusKind, SearchPlaneService};

#[tokio::test]
async fn studio_sql_flight_provider_supports_information_schema_tables() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaService", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    let epoch = publish_reference_hits(&service, "fp-sql-information-schema-1", &hits).await;
    let provider = StudioSqlFlightRouteProvider::new(service.clone());
    let _engine_table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::ReferenceOccurrence,
        epoch,
    );
    let sql_table_name = SearchCorpusKind::ReferenceOccurrence.to_string();

    let response = provider
        .sql_query_batches(
            format!(
                "SELECT table_name, table_type FROM information_schema.tables WHERE table_name IN ('{sql_table_name}', '{STUDIO_SQL_CATALOG_TABLE_NAME}', '{STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME}') ORDER BY table_name"
            )
            .as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("information_schema.tables query batches: {error}"));

    assert_eq!(response.batches.len(), 1);
    assert_eq!(response.batches[0].num_rows(), 3);
    assert_eq!(
        string_column_values(&response.batches[0], "table_name"),
        vec![
            sql_table_name,
            STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME.to_string(),
            STUDIO_SQL_CATALOG_TABLE_NAME.to_string(),
        ]
    );
    assert_eq!(
        string_column_values(&response.batches[0], "table_type"),
        vec![
            "BASE TABLE".to_string(),
            "BASE TABLE".to_string(),
            "BASE TABLE".to_string(),
        ]
    );
}

#[tokio::test]
async fn studio_sql_flight_provider_supports_information_schema_columns() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaService", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    let epoch = publish_reference_hits(&service, "fp-sql-information-schema-2", &hits).await;
    let provider = StudioSqlFlightRouteProvider::new(service.clone());
    let _engine_table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::ReferenceOccurrence,
        epoch,
    );
    let sql_table_name = SearchCorpusKind::ReferenceOccurrence.to_string();

    let response = provider
        .sql_query_batches(
            format!(
                "SELECT column_name FROM information_schema.columns WHERE table_name = '{sql_table_name}' ORDER BY ordinal_position"
            )
            .as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("information_schema.columns query batches: {error}"));

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
}
