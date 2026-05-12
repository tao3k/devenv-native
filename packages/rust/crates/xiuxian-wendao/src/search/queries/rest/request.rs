//! `search::queries::rest::request` owns Wendao queries rest request behavior.

use serde::{Deserialize, Serialize};

/// Request payload for the shared REST query adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query_language", rename_all = "snake_case")]
pub enum RestQueryRequest {
    /// Execute one SQL statement through the shared query system.
    Sql {
        /// SQL statement executed against the shared query system.
        query: String,
    },
    /// Execute one GraphQL document through the shared query system.
    Graphql {
        /// GraphQL document executed against the shared query system.
        document: String,
    },
}
