//! `search::queries::rest::response` owns Wendao queries rest response behavior.

use serde::{Deserialize, Serialize};

use crate::search::queries::graphql::GraphqlQueryPayload;
use crate::search::queries::sql::SqlQueryPayload;

/// Response payload for the shared REST query adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query_language", content = "payload", rename_all = "snake_case")]
pub enum RestQueryPayload {
    /// SQL response rendered by the shared query system.
    Sql(Box<SqlQueryPayload>),
    /// GraphQL response rendered by the shared query system.
    Graphql(GraphqlQueryPayload),
}
