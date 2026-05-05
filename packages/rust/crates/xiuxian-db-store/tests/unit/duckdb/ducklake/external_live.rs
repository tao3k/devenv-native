use std::path::Path;
use std::sync::Arc;

use ::duckdb::arrow::{
    array::{ArrayRef, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use tempfile::tempdir;

use super::{
    DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, DuckDbS3SecretConfig,
    DuckDbS3SecretProvider, DuckLakeAttachConfig, DuckLakeCatalog, DuckLakeDataPath,
    DuckLakeTableRef, append_ducklake_record_batches, attach_ducklake, build_duckdb_s3_secret_sql,
    must_ok, open_duckdb_connection,
};

#[test]
#[ignore = "requires externally provisioned PostgreSQL catalog and optional S3 data path"]
fn ducklake_external_postgres_catalog_live_probe() {
    let Some(postgres_dsn) = optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_POSTGRES_DSN") else {
        eprintln!(
            "skipping external DuckLake probe: XIUXIAN_DUCKLAKE_EXTERNAL_POSTGRES_DSN is not set"
        );
        return;
    };
    let Some(data_path) = optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_DATA_PATH") else {
        eprintln!(
            "skipping external DuckLake probe: XIUXIAN_DUCKLAKE_EXTERNAL_DATA_PATH is not set"
        );
        return;
    };

    let root = must_ok(tempdir(), "create DuckLake external probe root");
    let runtime = external_probe_runtime(root.path());
    let connection = must_ok(
        open_duckdb_connection(&runtime),
        "open DuckDB for external DuckLake probe",
    );
    if let Some(secret_config) = external_probe_s3_secret_config() {
        let secret_sql = must_ok(
            build_duckdb_s3_secret_sql(&secret_config),
            "external probe S3 secret SQL",
        );
        must_ok(
            connection.execute_batch(secret_sql.as_str()),
            "install external probe S3 secret",
        );
    }

    let alias = optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_ALIAS")
        .unwrap_or_else(|| "wendao_external_lake".to_string());
    let config = DuckLakeAttachConfig {
        alias: alias.clone(),
        catalog: DuckLakeCatalog::postgres_connection_string(postgres_dsn),
        data_path: external_probe_data_path(data_path),
        bootstrap_extensions: true,
    };
    must_ok(
        attach_ducklake(&connection, &config),
        "attach external DuckLake catalog",
    );

    let table_name = external_probe_table_name();
    must_ok(
        connection.execute_batch(
            format!("CREATE TABLE {alias}.{table_name} (probe_id VARCHAR, event_type VARCHAR);")
                .as_str(),
        ),
        "create external DuckLake probe table",
    );
    let batch = external_probe_batch(table_name.as_str());
    let appended_rows = must_ok(
        append_ducklake_record_batches(
            &connection,
            &DuckLakeTableRef::main_schema(alias.as_str(), table_name.as_str()),
            vec![batch],
        ),
        "append external DuckLake probe Arrow batch",
    );
    assert_eq!(appended_rows, 1);

    let event_count: i64 = must_ok(
        connection.query_row(
            format!(
                "SELECT COUNT(*) FROM {alias}.{table_name} WHERE event_type = 'external.probe'"
            )
            .as_str(),
            [],
            |row| row.get(0),
        ),
        "query external DuckLake probe rows",
    );
    assert_eq!(event_count, 1);

    must_ok(
        connection.execute_batch(format!("DROP TABLE {alias}.{table_name};").as_str()),
        "drop external DuckLake probe table",
    );
}

fn external_probe_runtime(root: &Path) -> DuckDbRuntimeConfig {
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

fn external_probe_batch(table_name: &str) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("probe_id", DataType::Utf8, false),
        Field::new("event_type", DataType::Utf8, false),
    ]));
    must_ok(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec![table_name])) as ArrayRef,
                Arc::new(StringArray::from(vec!["external.probe"])),
            ],
        ),
        "build external DuckLake probe Arrow batch",
    )
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn external_probe_data_path(value: String) -> DuckLakeDataPath {
    if value.contains("://") {
        DuckLakeDataPath::remote_uri(value)
    } else {
        DuckLakeDataPath::local(value)
    }
}

fn external_probe_s3_secret_config() -> Option<DuckDbS3SecretConfig> {
    let name = optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_SECRET_NAME")?;
    let provider = match (
        optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_KEY_ID"),
        optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_SECRET"),
    ) {
        (Some(key_id), Some(secret)) => DuckDbS3SecretProvider::Config {
            key_id,
            secret,
            session_token: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_SESSION_TOKEN"),
        },
        _ => DuckDbS3SecretProvider::CredentialChain {
            chain: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_CHAIN"),
        },
    };
    let mut config = DuckDbS3SecretConfig {
        name,
        provider,
        region: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_REGION"),
        endpoint: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_ENDPOINT"),
        url_style: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_URL_STYLE"),
        scope: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_SCOPE"),
        use_ssl: optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_S3_USE_SSL")
            .and_then(|value| value.parse().ok()),
        bootstrap_httpfs: true,
    };
    if config.scope.is_none() {
        config.scope = optional_env("XIUXIAN_DUCKLAKE_EXTERNAL_DATA_PATH");
    }
    Some(config)
}

fn external_probe_table_name() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("ducklake_external_probe_{}_{}", std::process::id(), nanos)
}
