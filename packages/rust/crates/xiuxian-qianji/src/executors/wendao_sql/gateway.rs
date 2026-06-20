use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::OnceLock;
use std::time::Duration;

/// Stable SQL query metadata mirrored from the Wendao REST surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SqlQueryMetadata {
    pub(crate) catalog_table_name: String,
    pub(crate) column_catalog_table_name: String,
    pub(crate) view_source_catalog_table_name: String,
    pub(crate) supports_information_schema: bool,
    pub(crate) registered_tables: Vec<String>,
    pub(crate) registered_table_count: usize,
    pub(crate) registered_view_count: usize,
    pub(crate) registered_column_count: usize,
    pub(crate) registered_view_source_count: usize,
    pub(crate) result_batch_count: usize,
    pub(crate) result_row_count: usize,
}

/// Stable SQL result-column description mirrored from the Wendao REST surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SqlColumnPayload {
    pub(crate) name: String,
    pub(crate) data_type: String,
    pub(crate) nullable: bool,
}

/// Stable SQL result batch mirrored from the Wendao REST surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SqlBatchPayload {
    pub(crate) row_count: usize,
    pub(crate) columns: Vec<SqlColumnPayload>,
    pub(crate) rows: Vec<Map<String, Value>>,
}

/// Stable SQL payload mirrored from the Wendao REST surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SqlQueryPayload {
    pub(crate) metadata: SqlQueryMetadata,
    pub(crate) batches: Vec<SqlBatchPayload>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "query_language", rename_all = "snake_case")]
enum RestQueryRequest {
    Sql { query: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "query_language", content = "payload", rename_all = "snake_case")]
enum RestQueryPayload {
    Sql(Box<SqlQueryPayload>),
    Graphql(Value),
}

pub(crate) async fn query_sql_endpoint(
    endpoint: &str,
    query: &str,
) -> Result<SqlQueryPayload, String> {
    let response = http_client()
        .post(endpoint)
        .json(&RestQueryRequest::Sql {
            query: query.to_string(),
        })
        .send()
        .await
        .map_err(|error| format!("failed to call Wendao query endpoint: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<response body unavailable>".to_string());
        return Err(format!("Wendao query endpoint returned {status}: {body}"));
    }

    match response
        .json::<RestQueryPayload>()
        .await
        .map_err(|error| format!("failed to decode Wendao query payload: {error}"))?
    {
        RestQueryPayload::Sql(payload) => Ok(*payload),
        RestQueryPayload::Graphql(graphql_payload) => Err(format!(
            "Wendao query endpoint returned a GraphQL payload for a SQL request: {graphql_payload}"
        )),
    }
}

fn http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .brotli(true)
            .deflate(true)
            .gzip(true)
            .zstd(true)
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}
