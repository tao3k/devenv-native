use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepositoryPluginConfig, julia_arrow_request_schema,
    julia_arrow_response_schema,
};

use super::fetch_plugin_arrow_score_rows_for_repository;
use crate::julia_plugin_test_support::contract::request_batch;
use crate::julia_plugin_test_support::official_examples::{
    reserve_real_service_port, spawn_real_wendaoarrow_service, wait_for_service_ready,
};

#[test]
fn julia_arrow_request_schema_uses_contract_columns() {
    let schema = julia_arrow_request_schema(3);

    assert_eq!(schema.field(0).name(), "doc_id");
    assert_eq!(schema.field(1).name(), "vector_score");
    assert_eq!(schema.field(2).name(), "embedding");
    assert_eq!(schema.field(3).name(), "query_embedding");
}

#[test]
fn julia_arrow_response_schema_optionally_includes_trace_id() {
    let base = julia_arrow_response_schema(false);
    let traced = julia_arrow_response_schema(true);

    assert_eq!(base.fields().len(), 3);
    assert_eq!(traced.fields().len(), 4);
    assert_eq!(traced.field(3).name(), "trace_id");
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn fetch_plugin_arrow_score_rows_for_repository_roundtrips_remote_scores() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let _service = spawn_real_wendaoarrow_service(port);
    wait_for_service_ready(&base_url)
        .await
        .unwrap_or_else(|error| {
            panic!("real WendaoArrow Flight service should become ready: {error}")
        });
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "flight_transport": {
                    "base_url": base_url,
                    "route": "/rerank",
                    "schema_version": "v1"
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let rows = fetch_plugin_arrow_score_rows_for_repository(&repository, &[request_batch()])
        .await
        .unwrap_or_else(|error| panic!("transport should succeed: {error}"));

    assert_eq!(rows.len(), 2);
    assert_eq!(rows.get("doc-a").map(|row| row.analyzer_score), Some(0.4));
    assert_eq!(rows.get("doc-a").map(|row| row.final_score), Some(0.4));
}
