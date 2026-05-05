use super::{
    DuckDbS3SecretConfig, DuckDbS3SecretProvider, DuckLakeAttachConfig, DuckLakeCatalog,
    DuckLakeDataPath, build_duckdb_s3_secret_sql, build_ducklake_attach_sql,
    build_ducklake_extension_bootstrap_sql, build_ducklake_use_sql, must_ok,
};

#[test]
fn ducklake_local_attach_sql_bootstraps_extension_and_escapes_paths() {
    let config = DuckLakeAttachConfig::local(
        "wendao_lake",
        "metadata/local's.ducklake",
        "data/wendao_lake's",
    );

    let sql = must_ok(
        build_ducklake_attach_sql(&config),
        "valid local DuckLake attach SQL",
    );

    assert!(sql.contains("INSTALL ducklake;"));
    assert!(sql.contains("LOAD ducklake;"));
    assert!(sql.contains(
        "ATTACH 'ducklake:metadata/local''s.ducklake' AS \"wendao_lake\" (DATA_PATH 'data/wendao_lake''s');"
    ));
}

#[test]
fn ducklake_postgres_attach_sql_loads_postgres_extension() {
    let config = DuckLakeAttachConfig::postgres(
        "wendao_lake",
        "dbname=ducklake_catalog host=localhost user=postgres",
        "data_files",
    );

    let bootstrap_sql = must_ok(
        build_ducklake_extension_bootstrap_sql(&config.catalog),
        "valid DuckLake Postgres bootstrap SQL",
    );
    assert!(bootstrap_sql.contains("INSTALL ducklake;"));
    assert!(bootstrap_sql.contains("LOAD ducklake;"));
    assert!(bootstrap_sql.contains("INSTALL postgres;"));
    assert!(bootstrap_sql.contains("LOAD postgres;"));

    let attach_sql = must_ok(
        build_ducklake_attach_sql(&config),
        "valid DuckLake Postgres attach SQL",
    );
    assert!(attach_sql.contains(
        "ATTACH 'ducklake:postgres:dbname=ducklake_catalog host=localhost user=postgres' AS \"wendao_lake\" (DATA_PATH 'data_files');"
    ));
}

#[test]
fn ducklake_postgres_remote_s3_attach_sql_keeps_remote_data_path_as_uri() {
    let config = DuckLakeAttachConfig::postgres_remote_data_path(
        "wendao_lake",
        "dbname=ducklake_catalog host=localhost user=postgres",
        "s3://wendao-lake/events/",
    );

    let attach_sql = must_ok(
        build_ducklake_attach_sql(&config),
        "valid DuckLake Postgres plus S3 data path attach SQL",
    );

    assert!(attach_sql.contains("INSTALL ducklake;"));
    assert!(attach_sql.contains("LOAD ducklake;"));
    assert!(attach_sql.contains("INSTALL postgres;"));
    assert!(attach_sql.contains("LOAD postgres;"));
    assert!(attach_sql.contains(
        "ATTACH 'ducklake:postgres:dbname=ducklake_catalog host=localhost user=postgres' AS \"wendao_lake\" (DATA_PATH 's3://wendao-lake/events/');"
    ));

    let invalid_uri = DuckLakeAttachConfig::postgres_remote_data_path(
        "wendao_lake",
        "dbname=ducklake_catalog host=localhost user=postgres",
        "ftp://bucket/events/",
    );
    assert!(build_ducklake_attach_sql(&invalid_uri).is_err());
}

#[test]
fn duckdb_s3_secret_sql_bootstraps_httpfs_and_escapes_values() {
    let config = DuckDbS3SecretConfig::config("wendao_s3", "key'id", "secret'value")
        .with_region("us-west-2")
        .with_endpoint("localhost:9000")
        .with_url_style("path")
        .with_scope("s3://wendao-lake/events/")
        .with_use_ssl(false);

    let sql = must_ok(
        build_duckdb_s3_secret_sql(&config),
        "valid DuckDB S3 secret SQL",
    );

    assert!(sql.contains("INSTALL httpfs;"));
    assert!(sql.contains("LOAD httpfs;"));
    assert!(sql.contains("CREATE OR REPLACE SECRET wendao_s3"));
    assert!(sql.contains("PROVIDER config"));
    assert!(sql.contains("KEY_ID 'key''id'"));
    assert!(sql.contains("SECRET 'secret''value'"));
    assert!(sql.contains("REGION 'us-west-2'"));
    assert!(sql.contains("ENDPOINT 'localhost:9000'"));
    assert!(sql.contains("URL_STYLE 'path'"));
    assert!(sql.contains("SCOPE 's3://wendao-lake/events/'"));
    assert!(sql.contains("USE_SSL false"));

    let chain_config = DuckDbS3SecretConfig {
        name: "wendao_s3_chain".to_string(),
        provider: DuckDbS3SecretProvider::CredentialChain {
            chain: Some("config".to_string()),
        },
        region: Some("us-east-1".to_string()),
        endpoint: None,
        url_style: None,
        scope: None,
        use_ssl: None,
        bootstrap_httpfs: false,
    };
    let chain_sql = must_ok(
        build_duckdb_s3_secret_sql(&chain_config),
        "valid DuckDB credential-chain S3 secret SQL",
    );
    assert!(!chain_sql.contains("INSTALL httpfs;"));
    assert!(chain_sql.contains("PROVIDER credential_chain"));
    assert!(chain_sql.contains("CHAIN config"));
}

#[test]
fn ducklake_attach_sql_validates_alias_catalog_and_data_path() {
    let invalid_alias = DuckLakeAttachConfig::local("9lake", "metadata.ducklake", "data_files");
    assert!(build_ducklake_attach_sql(&invalid_alias).is_err());

    let invalid_postgres = DuckLakeAttachConfig::postgres("wendao_lake", "   ", "data_files");
    assert!(build_ducklake_attach_sql(&invalid_postgres).is_err());

    let invalid_data_path = DuckLakeAttachConfig {
        alias: "wendao_lake".to_string(),
        catalog: DuckLakeCatalog::local_metadata_file("metadata.ducklake"),
        data_path: DuckLakeDataPath::local(std::path::PathBuf::new()),
        bootstrap_extensions: true,
    };
    assert!(build_ducklake_attach_sql(&invalid_data_path).is_err());

    let use_sql = must_ok(
        build_ducklake_use_sql("wendao_lake"),
        "valid DuckLake use SQL",
    );
    assert_eq!(use_sql, "USE \"wendao_lake\";");
}
