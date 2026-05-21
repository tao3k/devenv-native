//! `search::queries::graphql::execution` owns Wendao queries graphql execution behavior.

use serde_json::{Map, Value};

use crate::search::queries::SearchQueryService;
use crate::search::queries::graphql::context::GraphqlExecutionContext;
use crate::search::queries::graphql::document::parse_graphql_document;
use crate::search::queries::graphql::payload::GraphqlQueryPayload;
use crate::search::queries::graphql::translation::build_graphql_sql_query;
use crate::search::queries::sql::execute_sql_query;

/// Execute one `GraphQL` document against the shared request-scoped SQL surface.
///
/// # Errors
///
/// Returns an error when the `GraphQL` document cannot be parsed or when the
/// shared request-scoped query surface cannot be planned, executed, or
/// serialized.
pub async fn query_graphql_payload(
    service: &SearchQueryService,
    document: &str,
) -> Result<GraphqlQueryPayload, String> {
    let context = GraphqlExecutionContext::new().with_query_service(service.clone());
    query_graphql_payload_with_context(&context, document).await
}

pub(crate) async fn query_graphql_payload_with_context(
    context: &GraphqlExecutionContext,
    document: &str,
) -> Result<GraphqlQueryPayload, String> {
    let query = parse_graphql_document(document)?;
    let query_text = build_graphql_sql_query(&query)
        .map_err(|error| format!("GraphQL SQL translation failed: {error}"))?;
    let value =
        execute_table_query(context, query_text.as_str(), query.response_key.as_str()).await?;
    let mut data = Map::new();
    data.insert(query.response_key, value);
    Ok(GraphqlQueryPayload { data })
}

async fn execute_table_query(
    context: &GraphqlExecutionContext,
    query_text: &str,
    response_key: &str,
) -> Result<Value, String> {
    let Some(query_service) = context.query_service() else {
        return Err("GraphQL queries require a shared query service".to_string());
    };

    let (_metadata, batches) = execute_sql_query(query_service, query_text)
        .await
        .map_err(|error| {
            format!(
                "GraphQL table query `{response_key}` failed through the shared SQL surface: {error}"
            )
        })?
        .into_parts();
    let rows = crate::search::queries::sql::engine_batches_rows_payload(batches.as_slice())?;
    Ok(Value::Array(rows.into_iter().map(Value::Object).collect()))
}
