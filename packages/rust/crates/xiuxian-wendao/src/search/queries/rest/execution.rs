//! `search::queries::rest::execution` owns Wendao queries rest execution behavior.

use crate::search::queries::SearchQueryService;
use crate::search::queries::graphql::query_graphql_payload;
use crate::search::queries::sql::query_sql_payload;

use super::request::RestQueryRequest;
use super::response::RestQueryPayload;

/// Execute one REST-style query request against the shared query system.
///
/// # Errors
///
/// Returns an error when the delegated SQL or GraphQL adapter cannot validate,
/// plan, execute, or serialize the request payload.
pub async fn query_rest_payload(
    service: &SearchQueryService,
    request: &RestQueryRequest,
) -> Result<RestQueryPayload, String> {
    match request {
        RestQueryRequest::Sql { query } => query_sql_payload(service, query)
            .await
            .map(Box::new)
            .map(RestQueryPayload::Sql),
        RestQueryRequest::Graphql { document } => query_graphql_payload(service, document)
            .await
            .map(RestQueryPayload::Graphql),
    }
}
